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
use log::kv::source;
use ruzzic_common::{
    read_bytes_to::{FromReadBytes, FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    EndpointType, QuicVersion,
};
use sha2::{Digest, Sha256};

use crate::{
    connection::{Connection, ConnectionID},
    endpoint_state::EndpointState,
    frame::Frames,
    size_of_varint, Version,
};

use self::{long_header::LongHeader, packet_meta::PacketMeta};

mod long_header;
pub mod packet_meta;

#[derive(Debug, PartialEq)]
pub struct Packet {
    meta: PacketMeta,
    // Rather than Packet Body, something other than Packet Meta is the correct expression.
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
        let packet_number_length = self.meta.packet_number_length();
        let packet_body_length = self.body.raw_length(Some(packet_number_length as usize));
        let packet_meta_length = self.meta.raw_length();
        let packet_payload_length = self.payload().len();
        let header_length = packet_meta_length + (packet_body_length - packet_payload_length);
        self.raw[..header_length].to_owned()
    }

    fn update_payload(self, payload: PacketPayload) -> Self {
        Self {
            meta: self.meta,
            body: self.body.update_payload(payload),
            raw: self.raw,
        }
    }

    fn new_initial(connection: &Connection, endpoint_state: &EndpointState) -> Self {
        let unprotected_packet = Self::new_unprotected_initial(connection, endpoint_state);
        unprotected_packet.encrypt(connection, endpoint_state)
    }

    fn new_unprotected_initial(connection: &Connection, endpoint_state: &EndpointState) -> Self {
        let meta = PacketMeta::new_initial(connection, endpoint_state);
        let body = PacketBody::new_initial(connection, endpoint_state);
        let raw = Vec::new();
        Self { meta, body, raw }
    }

    pub fn decrypt(&self, endpoint_state: &EndpointState) -> Self {
        let initial_salt = Into::<QuicVersion>::into(self.version()).initial_salt();

        let decryption_kit = DecryptionKit::new(
            &initial_salt,
            self.client_id(endpoint_state),
            endpoint_state.type_is(),
        );
        log::debug!("Decryption kit: {:x?}", decryption_kit);

        let header_removal_kit = HeaderRemovalKit::new(
            self.destination_connection_id(),
            self.source_connection_id(),
            self.payload(),
            &self.body,
            &self.raw,
            &decryption_kit.hp,
            128 / 8, // AES_128_GCM key length
        );

        let unprotected_packet = header_removal_kit.remove_protection(&self.raw);

        let packet_number = unprotected_packet.body.packet_number();
        let packet_number_bytes = packet_number.0.to_be_bytes();
        let packet_header = unprotected_packet.get_header_bytes();

        let decrypted_payload = decrypt_payload(
            unprotected_packet.payload(),
            &packet_number_bytes,
            decryption_kit,
            &packet_header,
        );

        unprotected_packet.update_payload(PacketPayload::from_vec(decrypted_payload))
    }

    fn encrypt(self, connection: &Connection, endpoint_state: &EndpointState) -> Self {
        todo!()
    }

    pub(crate) fn client_id(&self, endpoint_state: &EndpointState) -> &ConnectionID {
        match endpoint_state.type_is() {
            EndpointType
        }
    }
}

fn decrypt_payload(
    payload: &[u8],
    packet_number: &[u8],
    decryption_kit: DecryptionKit,
    packet_header: &[u8],
) -> Vec<u8> {
    let packet_number = [
        &vec![0; decryption_kit.iv.len() - packet_number.len()][..],
        &packet_number,
    ]
    .concat();
    let nonce = packet_number
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ decryption_kit.iv[i])
        .collect::<Vec<u8>>();

    let aad = packet_header;
    let msg = payload;

    let aead_payload = aead::Payload { msg, aad };
    // Only considered during InitialPacket
    let aes = Aes128Gcm::new(&GenericArray::clone_from_slice(&decryption_kit.key));

    // TODO: erroro handling
    match aes.decrypt(&GenericArray::from_slice(&nonce), aead_payload) {
        Err(_) => {
            panic!("Crypto Error. Details are not displayed to prevent attacks using error information.")
        }
        Ok(result) => result,
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

#[derive(Debug)]
struct DecryptionKit {
    pub(self) key: Vec<u8>,
    pub(self) iv: Vec<u8>,
    pub(self) hp: Vec<u8>,
}

impl DecryptionKit {
    fn new(
        initial_salt: &[u8],
        connection_id: &ConnectionID,
        endpoint_type: &EndpointType,
    ) -> Self {
        let (key, iv, hp) = match endpoint_type {
            EndpointType::Server => {
                // server received a client packet
                new_decryption_kit(initial_salt, connection_id, "client in".as_bytes())
            }
            EndpointType::Client => {
                // client received a server packet
                new_decryption_kit(initial_salt, connection_id, "server in".as_bytes())
            }
        };

        Self { key, iv, hp }
    }
}

fn new_decryption_kit(
    initial_salt: &[u8],
    connection_id: &ConnectionID,
    label: &[u8],
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    log::debug!("initial_salt: {initial_salt:x?}");
    log::debug!("connection_id: {connection_id:x?}",);
    let initial_secret = hkdf_extract(&initial_salt, &connection_id.to_vec());
    log::debug!("initial_secret: {initial_secret:x?}");

    let endpoint_initial_secret =
        hkdf_expand_label(&initial_secret, label, &[], Sha256::output_size() as u16);

    get_key_iv_hp(&endpoint_initial_secret)
}

#[derive(Debug)]
struct HeaderRemovalKit {
    pub(self) packet_number_offset: usize,
    pub(self) packet_number_length: usize,
    pub(self) mask: Vec<u8>,
    first_byte: u8,
}

impl HeaderRemovalKit {
    fn new(
        destination_connection_id: ConnectionID,
        source_connection_id: Option<ConnectionID>,
        payload: &[u8],
        body: &PacketBody,
        raw: &[u8],
        hp: &[u8],
        sample_length: usize,
    ) -> Self {
        let packet_number_offset = match body {
            PacketBody::Long(lh) => {
                7
                + destination_connection_id.len()
                // long header packet must have a source connection id
                + source_connection_id.unwrap().len()
                + size_of_varint(payload.len() as u64)
                + match lh {
                    LongHeader::Initial(b) => b.token_raw_length(),
                    _ => 0,
                }
            }
            // short header packet
            _ => 1 + destination_connection_id.len(),
        };

        let sample_offset = 4 + packet_number_offset;
        let sample = &raw[sample_offset..sample_offset + sample_length];

        let mask = {
            let mut mask = GenericArray::clone_from_slice(&sample);
            Aes128::new_from_slice(hp).unwrap().encrypt_block(&mut mask);
            mask
        }
        .to_vec();

        let unprotected_first_byte = match body {
            PacketBody::Long(_) => raw[0] ^ (mask[0] & 0x0f),
            // short header packet
            _ => raw[0] ^ (mask[0] & 0x1f),
        };

        let packet_number_length = (unprotected_first_byte & 0x03) as usize + 1;

        Self {
            packet_number_offset,
            packet_number_length,
            mask: mask,
            first_byte: unprotected_first_byte,
        }
    }

    fn remove_protection(&self, raw: &[u8]) -> Packet {
        let mut raw = raw.to_vec();

        let packet_number = raw
            [self.packet_number_offset..self.packet_number_offset + self.packet_number_length]
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ self.mask[i + 1])
            .collect::<Vec<u8>>();

        raw[self.packet_number_offset..self.packet_number_offset + self.packet_number_length]
            .copy_from_slice(&packet_number);
        raw[0] = self.first_byte;

        let mut input = Cursor::new(raw);
        // this must be succeeded
        Packet::from_read_bytes(&mut input).unwrap()
    }
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

    fn raw_length(&self, packet_number_length: Option<usize>) -> usize {
        match self {
            PacketBody::Long(lh) => lh.raw_length(packet_number_length),
            _ => unimplemented!(),
        }
    }

    fn destination_connection_id(&self) -> ConnectionID {
        match self {
            PacketBody::Long(lh) => lh.destination_connection_id(),
            _ => unimplemented!(),
        }
    }

    fn update_payload(self, payload: PacketPayload) -> Self {
        match self {
            PacketBody::Long(lh) => PacketBody::Long(lh.update_payload(payload)),
            _ => unimplemented!(),
        }
    }

    fn new_initial(connection: &Connection, endpoint_state: &EndpointState) -> Self {
        Self::Long(LongHeader::new_initial(connection, endpoint_state))
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

    pub(crate) fn length_in_header(&self) -> u8 {
        if self.0 < 0xff {
            0
        } else if self.0 < 0xffff {
            1
        } else if self.0 < 0xffffffff {
            2
        } else if self.0 < 1 << 62 - 1 {
            3
        } else {
            unreachable!(
                "maximum packet number size is 2^62 - 1 but this is {}",
                self.0
            )
        }
    }

    pub(crate) fn zero() -> Self {
        Self(0)
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
    fn raw_length(&self, packet_number_length: usize) -> usize {
        let Self {
            packet_number,
            packet_payload,
        } = self;
        let packet_data_length = packet_payload.0.len() + packet_number_length;
        size_of_varint(packet_data_length as u64) + packet_data_length
    }
}

#[cfg(test)]
mod neqo_tests;

#[cfg(test)]
mod rfc9000_tests;
impl PacketPayload {
    pub(crate) fn from_vec(vec: Vec<u8>) -> PacketPayload {
        PacketPayload(vec)
    }
}
