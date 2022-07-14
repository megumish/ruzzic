use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith};

use crate::handshake::HandshakeType;

mod application_layer_protocol_negotiation;
mod certificate_authorities;
mod client_certificate_type;
mod cookie;
mod early_data;
mod heartbeat;
mod key_share;
mod max_fragment_length;
mod oid_filters;
mod others;
mod padding;
mod post_handshake_auth;
mod pre_shared_key;
mod psk_key_exchange_modes;
mod renegotiation_info;
mod server_certificate_type;
mod server_name;
mod signature_algorithms;
mod signature_algorithms_cert;
mod signed_certificate_timestamp;
mod status_request;
mod supported_groups;
mod supported_versions;
mod use_srtp;

#[derive(Debug)]
pub enum Extension {
    ServerName(server_name::Body),
    MaxFragmentLength(max_fragment_length::Body),
    StatusRequest(status_request::Body),
    SupportedGroups(supported_groups::Body),
    SignatureAlgorithms(signature_algorithms::Body),
    UseSrtp(use_srtp::Body),
    Heartbeat(heartbeat::Body),
    ApplicationLayerProtocolNegotiation(application_layer_protocol_negotiation::Body),
    SignedCertificateTimestamp(signed_certificate_timestamp::Body),
    ClientCertificateType(client_certificate_type::Body),
    ServerCertificateType(server_certificate_type::Body),
    Padding(padding::Body),
    PreSharedKey(pre_shared_key::Body),
    EarlyData(early_data::Body),
    SupportedVersions(supported_versions::Body),
    Cookie(cookie::Body),
    PskKeyExchangeModes(psk_key_exchange_modes::Body),
    CertificateAuthorities(certificate_authorities::Body),
    OidFilters(oid_filters::Body),
    PostHandshakeAuth(post_handshake_auth::Body),
    SignatureAlgorithmsCert(signature_algorithms_cert::Body),
    KeyShare(key_share::Body),
    RenegotiationInfo(renegotiation_info::Body),
    Others(others::Body),
}

pub type Extensions = Vec<Extension>;

impl FromReadBytesWith<HandshakeType> for Extension {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        handshake_type: HandshakeType,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let extension_type = input.read_u16::<NetworkEndian>()?;
        Ok(match extension_type {
            0x00 => Extension::ServerName(input.read_bytes_to()?),
            0x01 => Extension::MaxFragmentLength(input.read_bytes_to()?),
            0x05 => Extension::StatusRequest(input.read_bytes_to()?),
            0x0a => Extension::SupportedGroups(input.read_bytes_to()?),
            0x0d => Extension::SignatureAlgorithms(input.read_bytes_to()?),
            0x0e => Extension::UseSrtp(input.read_bytes_to()?),
            0x0f => Extension::Heartbeat(input.read_bytes_to()?),
            0x10 => Extension::ApplicationLayerProtocolNegotiation(input.read_bytes_to()?),
            0x12 => Extension::SignedCertificateTimestamp(input.read_bytes_to()?),
            0x13 => Extension::ClientCertificateType(input.read_bytes_to()?),
            0x14 => Extension::ServerCertificateType(input.read_bytes_to()?),
            0x15 => Extension::Padding(input.read_bytes_to()?),
            0x29 => Extension::PreSharedKey(input.read_bytes_to()?),
            0x2a => Extension::EarlyData(input.read_bytes_to_with(handshake_type)?),
            0x2b => Extension::SupportedVersions(input.read_bytes_to()?),
            0x2c => Extension::Cookie(input.read_bytes_to()?),
            0x2d => Extension::PskKeyExchangeModes(input.read_bytes_to()?),
            0x2f => Extension::CertificateAuthorities(input.read_bytes_to()?),
            0x30 => Extension::OidFilters(input.read_bytes_to()?),
            0x31 => Extension::PostHandshakeAuth(input.read_bytes_to()?),
            0x32 => Extension::SignatureAlgorithmsCert(input.read_bytes_to()?),
            0x33 => Extension::KeyShare(input.read_bytes_to()?),
            0xff01 => Extension::RenegotiationInfo(input.read_bytes_to()?),
            _ => Extension::Others(input.read_bytes_to()?),
        })
    }
}

impl Extension {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.body_size()
    }

    fn body_size(&self) -> usize {
        match self {
            Extension::ServerName(b) => b.size_of(),
            Extension::MaxFragmentLength(b) => b.size_of(),
            Extension::StatusRequest(b) => b.size_of(),
            Extension::SupportedGroups(b) => b.size_of(),
            Extension::SignatureAlgorithms(b) => b.size_of(),
            Extension::UseSrtp(b) => b.size_of(),
            Extension::Heartbeat(b) => b.size_of(),
            Extension::ApplicationLayerProtocolNegotiation(b) => b.size_of(),
            Extension::SignedCertificateTimestamp(b) => b.size_of(),
            Extension::ClientCertificateType(b) => b.size_of(),
            Extension::ServerCertificateType(b) => b.size_of(),
            Extension::Padding(b) => b.size_of(),
            Extension::PreSharedKey(b) => b.size_of(),
            Extension::EarlyData(b) => b.size_of(),
            Extension::SupportedVersions(b) => b.size_of(),
            Extension::Cookie(b) => b.size_of(),
            Extension::PskKeyExchangeModes(b) => b.size_of(),
            Extension::CertificateAuthorities(b) => b.size_of(),
            Extension::OidFilters(b) => b.size_of(),
            Extension::PostHandshakeAuth(b) => b.size_of(),
            Extension::SignatureAlgorithmsCert(b) => b.size_of(),
            Extension::KeyShare(b) => b.size_of(),
            Extension::RenegotiationInfo(b) => b.size_of(),
            Extension::Others(b) => b.size_of(),
        }
    }
}
