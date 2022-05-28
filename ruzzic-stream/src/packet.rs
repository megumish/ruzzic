use std::{borrow::Cow, io::Cursor, iter};

use aes_gcm::{
    aead::{self, Aead, NewAead},
    aes::{Aes128, BlockEncrypt, NewBlockCipher},
    Aes128Gcm,
};
use bitvec::{field::BitField, macros::internal::funty::IsNumber};
use byteorder::{BigEndian, ByteOrder};
use generic_array::GenericArray;
use hmac::{Hmac, Mac};
use ruzzic_common::QuicVersion;
use sha2::{Digest, Sha256};

use crate::{
    connection::ConnectionID,
    frame::Frames,
    read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    size_of_varint, FromReadBytes, Version,
};

use self::{long_header::LongHeader, packet_meta::PacketMeta};

mod long_header;
pub mod packet_meta;

#[derive(Debug, PartialEq)]
pub struct Packet {
    meta: PacketMeta,
    body: PacketBody,
    raw: Vec<u8>,
}

impl FromReadBytesWith<()> for Packet {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut raw = Vec::new();
        input.read_to_end(&mut raw)?;
        let input = &mut Cursor::new(raw.clone());
        let meta = input.read_bytes_to()?;
        let body = input.read_bytes_to_with(&meta)?;
        Ok(Self { meta, body, raw })
    }
}

impl Packet {
    pub fn payload(&self) -> &[u8] {
        self.body.payload()
    }

    pub fn raw_length(&self) -> usize {
        self.raw.len()
    }

    pub fn version(&self) -> Version {
        self.meta.version()
    }

    pub fn destination_connection_id(&self) -> ConnectionID {
        match &self.body {
            PacketBody::Long(b) => b.destination_connection_id(),
            _ => unimplemented!(),
        }
    }

    pub fn source_connection_id(&self) -> Option<ConnectionID> {
        match &self.body {
            PacketBody::Long(b) => Some(b.source_connection_id()),
            _ => unimplemented!(),
        }
    }

    fn get_header_bytes(&self) -> Vec<u8> {
        self.raw[..self.raw_length() - self.payload().len()].to_vec()
    }

    pub fn remove_header_protection(&self) -> Packet {
        let initial_salt = Into::<QuicVersion>::into(self.version()).initial_salt();
        log::debug!("initial_salt: {initial_salt:x?}");
        let initial_secret =
            hkdf_extract(&initial_salt, &self.destination_connection_id().to_vec());
        log::debug!("initial_secret: {initial_secret:x?}");

        let client_initial_secret = hkdf_expand_label(
            &initial_secret,
            "client in".as_bytes(),
            &[],
            Sha256::output_size() as u16,
        );
        log::debug!("client_initial_secret: {client_initial_secret:x?}");
        let (client_key, client_iv, client_hp) = get_key_iv_hp(&client_initial_secret);
        log::debug!("client_key: {client_key:x?}");
        log::debug!("client_iv: {client_iv:x?}");
        log::debug!("client_hp: {client_hp:x?}");

        let server_initial_secret = hkdf_expand_label(
            &initial_secret,
            "server in".as_bytes(),
            &[],
            Sha256::output_size() as u16,
        );
        log::debug!("server_initial_secret: {server_initial_secret:x?}");
        let (server_key, server_iv, server_hp) = get_key_iv_hp(&server_initial_secret);
        log::debug!("server_key: {server_key:x?}");
        log::debug!("server_iv: {server_iv:x?}");
        log::debug!("server_hp: {server_hp:x?}");

        // TODO: this is long header packet only
        let pn_offset = 7
            + self.destination_connection_id().len()
            + self.source_connection_id().unwrap().len()
            + size_of_varint(self.payload().len() as u64)
            + match &self.body {
                PacketBody::Long(LongHeader::Initial(b)) => b.token_raw_length(),
                _ => 0,
            };
        log::debug!("pn_offset: {pn_offset}");
        let sample_offset = pn_offset + 4;
        log::debug!("sample_offset: {sample_offset}");
        let sample_length = 16; // aes_gcm_128_key_size
        log::debug!("sample_length: {sample_length}");

        let sample = &self.raw[sample_offset..sample_offset + sample_length];
        log::debug!("sample: {sample:x?}");

        let mask = {
            let mut mask = GenericArray::clone_from_slice(sample);
            Aes128::new(&GenericArray::clone_from_slice(&client_hp)).encrypt_block(&mut mask);
            mask[0..5].to_vec()
        };
        log::debug!("mask: {mask:x?}");

        // TODO: this is long header packet only
        let unprotected_first_byte = self.meta.first_byte.0.load_be::<u8>() ^ (mask[0] & 0x0f);
        log::debug!("unprotected_first_byte: {unprotected_first_byte:x?}");
        let mut unprotected_meta = self.meta.clone();
        unprotected_meta
            .first_byte
            .0
            .store_be(unprotected_first_byte);

        let unprotected_packet_length = unprotected_meta.packet_number_length();
        log::debug!("unprotected_packet_length: {unprotected_packet_length}");

        let mut raw = self.raw.clone();

        let unprotected_packet_number = raw
            [pn_offset..pn_offset + unprotected_packet_length as usize]
            .to_vec()
            .into_iter()
            .enumerate()
            .map(|(i, b)| b ^ mask[i + 1])
            .collect::<Vec<u8>>();
        raw[pn_offset..pn_offset + unprotected_packet_length as usize]
            .copy_from_slice(&unprotected_packet_number);
        raw[0] = unprotected_first_byte;
        let mut unprotected_input = Cursor::new(raw);

        let unprotected_packet = Packet::from_read_bytes(&mut unprotected_input).unwrap();

        let packet_number = unprotected_packet.body.packet_number();

        let packet_number_bytes = packet_number.0.to_be_bytes();
        let packet_number_bytes = [
            &vec![0; client_iv.len() - packet_number_bytes.len()][..],
            &packet_number_bytes,
        ]
        .concat();
        let nonce = packet_number_bytes
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ client_iv[i])
            .collect::<Vec<u8>>();

        let aad = unprotected_packet.get_header_bytes();
        log::debug!("aad: {aad:x?}");

        let data = unprotected_packet.payload();
        log::debug!("data: {data:x?}");

        let aes = Aes128Gcm::new(&GenericArray::clone_from_slice(&client_key));

        let aead_payload = aead::Payload {
            msg: data,
            aad: &aad,
        };
        let decrypted = aes
            .decrypt(&GenericArray::clone_from_slice(&nonce), aead_payload)
            .unwrap();

        let mut frames_input = Cursor::new(decrypted);
        let frames: Frames = frames_input.read_bytes_to().unwrap();
        log::debug!("frames: {frames:?}");
        todo!()
    }
}

fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(salt).expect("HMAC can take a key of any size");
    mac.update(ikm);
    mac.finalize().into_bytes().to_vec()
}

fn hkdf_expand(prk: &[u8], info: &[u8], length: u16) -> Vec<u8> {
    // TODO: uncheck
    let length_of_okm = div_ceil(length as usize, sha2::Sha256::output_size()) as u16;
    let mut okm = Vec::new();
    let mut t_result = Vec::new();
    for i in 1..length_of_okm + 1 {
        let mut t = Hmac::<Sha256>::new_from_slice(prk).expect("HMAC can take a key of any size");
        t.update(&t_result);
        t.update(info);
        t.update(&[i as u8]);
        t_result = t.finalize().into_bytes().to_vec();
        okm.append(&mut t_result.clone());
    }
    okm[..length as usize].to_vec()
}

fn hkdf_expand_label(secret: &[u8], label: &[u8], context: &[u8], length: u16) -> Vec<u8> {
    let length = length;
    let label = ["tls13 ".as_bytes(), label].concat();
    let hkdf_label = [
        &length.to_be_bytes()[..],
        &[label.len() as u8],
        &label,
        &[context.len() as u8],
        context,
    ]
    .concat();
    hkdf_expand(secret, &hkdf_label, length)
}

fn get_key_iv_hp(endpoint_initial_secret: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let key = hkdf_expand_label(
        endpoint_initial_secret,
        "quic key".as_bytes(),
        &[],
        16, // aes_gcm_128_key_size
    );
    let iv = hkdf_expand_label(
        endpoint_initial_secret,
        "quic iv".as_bytes(),
        &[],
        12, // aes_gcm_128_iv_size
    );
    let hp = hkdf_expand_label(
        endpoint_initial_secret,
        "quic hp".as_bytes(),
        &[],
        16, // aes_gcm_128_key_size
    );
    (key, iv, hp)
}

fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[derive(Debug, Clone, PartialEq)]
pub enum PacketBody {
    Long(LongHeader),
}

impl PacketBody {
    fn payload(&self) -> &[u8] {
        match self {
            PacketBody::Long(b) => b.payload(),
            _ => unimplemented!(),
        }
    }

    fn packet_number(&self) -> PacketNumber {
        match self {
            PacketBody::Long(b) => b.packet_number(),
            _ => unimplemented!(),
        }
    }

    fn raw_length(&self) -> usize {
        match self {
            PacketBody::Long(lh) => lh.raw_length(),
            _ => unimplemented!(),
        }
    }

    fn destination_connection_id(&self) -> ConnectionID {
        match self {
            PacketBody::Long(lh) => lh.destination_connection_id(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketBodyType {
    Long,
    Short,
}

impl FromReadBytesWith<&PacketMeta> for PacketBody {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        meta: &PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        match meta.get_type() {
            PacketBodyType::Long => Ok(PacketBody::Long(input.read_bytes_to_with(meta)?)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketNumber(pub(crate) u32);

impl PacketNumber {
    // TODO: check integer casting
    pub fn read_bytes_to(
        input: &mut impl std::io::Read,
        length: u8,
    ) -> Result<Self, std::io::Error> {
        let mut buf = vec![0u8; length as usize];
        input.read_exact(&mut buf)?;
        Ok(PacketNumber(
            BigEndian::read_uint(&buf, length as usize) as u32
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketPayload(Vec<u8>);

impl FromReadBytesWith<()> for PacketPayload {
    fn from_read_bytes_with<T: std::io::Read>(input: &mut T, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        Ok(PacketPayload(buf))
    }
}

struct PacketData<'a> {
    pub packet_number: &'a PacketNumber,
    pub packet_payload: &'a PacketPayload,
}

impl<'a> PacketData<'a> {
    fn raw_length(&self) -> usize {
        let Self {
            packet_number,
            packet_payload,
        } = self;
        let packet_data_length = packet_payload.0.len()
            + if packet_number.0 <= 0xff {
                1
            } else if packet_number.0 <= 0xffff {
                2
            } else {
                4
            };
        size_of_varint(packet_data_length as u64) + packet_data_length
    }
}

#[cfg(test)]
mod neqo_tests;

#[cfg(test)]
mod rfc9000_tests;
