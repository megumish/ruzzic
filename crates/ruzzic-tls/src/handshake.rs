use byteorder::ReadBytesExt;
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith};

mod certificate;
mod certificate_request;
mod certificate_verify;
mod client_hello;
mod encrypted_extensions;
mod finished;
mod key_update;
mod new_session_ticket;
mod others;
mod server_hello;

#[derive(Debug, PartialEq)]
pub enum Handshake {
    ClientHello(client_hello::Body),
    ServerHello(server_hello::Body),
    NewSessionTicket(new_session_ticket::Body),
    EndOfEarlyData,
    EncryptedExtensions(encrypted_extensions::Body),
    Certificate(certificate::Body),
    CertificateRequest(certificate_request::Body),
    CertificateVerify(certificate_verify::Body),
    Finished(finished::Body),
    KeyUpdate(key_update::Body),
    MessageHash,
    Others(others::Body),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeType {
    ClientHello,
    ServerHello,
    NewSessionTicket,
    EndOfEarlyData,
    EncryptedExtensions,
    Certificate,
    CertificateRequest,
    CertificateVerify,
    Finished,
    KeyUpdate,
    MessageHash,
    Others,
}

#[derive(Debug, PartialEq)]
pub(crate) struct HandshakeContext {
    pub handshake_type: HandshakeType,
}

impl FromReadBytesWith<()> for Handshake {
    fn from_read_bytes_with<T: std::io::Read>(input: &mut T, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let handshake_type = input.read_bytes_to()?;
        Ok(match handshake_type {
            HandshakeType::ClientHello => {
                Handshake::ClientHello(input.read_bytes_to_with(handshake_type)?)
            }
            HandshakeType::ServerHello => {
                Handshake::ServerHello(input.read_bytes_to_with(handshake_type)?)
            }
            HandshakeType::NewSessionTicket => Handshake::NewSessionTicket(input.read_bytes_to()?),
            HandshakeType::EndOfEarlyData => Handshake::EndOfEarlyData,
            HandshakeType::EncryptedExtensions => {
                Handshake::EncryptedExtensions(input.read_bytes_to()?)
            }
            HandshakeType::Certificate => Handshake::Certificate(input.read_bytes_to()?),
            HandshakeType::CertificateRequest => {
                Handshake::CertificateRequest(input.read_bytes_to_with(handshake_type)?)
            }
            HandshakeType::CertificateVerify => {
                Handshake::CertificateVerify(input.read_bytes_to()?)
            }
            HandshakeType::Finished => Handshake::Finished(input.read_bytes_to()?),
            HandshakeType::KeyUpdate => Handshake::KeyUpdate(input.read_bytes_to()?),
            HandshakeType::MessageHash => Handshake::MessageHash,
            _ => Handshake::Others(input.read_bytes_to()?),
        })
    }
}

impl FromReadBytesWith<()> for HandshakeType {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let handshake_type = input.read_u8()?;
        Ok(match handshake_type {
            0x01 => HandshakeType::ClientHello,
            0x02 => HandshakeType::ServerHello,
            0x04 => HandshakeType::NewSessionTicket,
            0x05 => HandshakeType::EndOfEarlyData,
            0x08 => HandshakeType::EncryptedExtensions,
            0x0b => HandshakeType::Certificate,
            0x0d => HandshakeType::CertificateRequest,
            0x0f => HandshakeType::CertificateVerify,
            0x14 => HandshakeType::Finished,
            0x18 => HandshakeType::KeyUpdate,
            0xfe => HandshakeType::MessageHash,
            _ => HandshakeType::Others,
        })
    }
}

#[cfg(test)]
mod rfc9000_tests;
