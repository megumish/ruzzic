use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

pub mod extension;
pub mod handshake;

use crate::extension::Extensions;

#[derive(Debug, PartialEq)]
enum CipherSuite {
    TlsAes128GcmSha256,
    TlsAes256GcmSha384,
    TlsChacha20Poly1305Sha256,
    TlsAes128CcmSha256,
    TlsAes128Ccm8Sha256,
    Others(u16),
}

impl FromReadBytesWith<()> for CipherSuite {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let value = input.read_u16::<NetworkEndian>()?;
        Ok(match value {
            0x1301 => CipherSuite::TlsAes128GcmSha256,
            0x1302 => CipherSuite::TlsAes256GcmSha384,
            0x1303 => CipherSuite::TlsChacha20Poly1305Sha256,
            0x1304 => CipherSuite::TlsAes128CcmSha256,
            0x1305 => CipherSuite::TlsAes128Ccm8Sha256,
            _ => CipherSuite::Others(value),
        })
    }
}

impl CipherSuite {
    pub(crate) fn size_of() -> usize {
        2
    }
}

#[derive(Debug, PartialEq)]
struct CertificateEntry {
    certificate: Certificate,
    extension: Extensions,
}

#[derive(Debug, PartialEq)]
enum Certificate {
    X509 {
        cert_data: Vec<u8>,
    },
    OpenPgpReserved,
    RawPublicKey {
        ans1_subject_public_key_info: Vec<u8>,
    },
}

#[derive(Debug, PartialEq)]
pub enum CertificateType {
    X509,
    OpenPgpReserved,
    RawPublicKey,
}

#[derive(Debug, PartialEq)]
enum SignatureScheme {
    // RSASSA-PKCS1-v1_5
    RsaPkcs1Sha256,
    RsaPkcs1Sha384,
    RsaPkcs1Sha512,
    // ECDSA
    EcdsaSecp256r1Sha256,
    EcdsaSecp384r1Sha384,
    EcdsaSecp521r1Sha512,
    // RSASSA-PSS with public key OID rsaEncryption
    RsaPssRsaeSha256,
    RsaPssRsaeSha384,
    RsaPssRsaeSha512,
    // EdDSA
    ed25519,
    ed448,
    // RSASSA-PSS with public key OID RSASSA-PSS
    RsaPssPssSha256,
    RsaPssPssSha384,
    RsaPssPssSha512,
    // Legacy
    RsaPkcs1Sha1,
    EcdsaSha1,
    // Reserved
    PrivateUse(u16),

    Others(u16),
}

#[derive(Debug, PartialEq)]
struct LegacyVersion(u16);

impl FromReadBytesWith<()> for LegacyVersion {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(LegacyVersion(input.read_u16::<NetworkEndian>()?))
    }
}

impl LegacyVersion {
    fn is_tls12(&self) -> bool {
        self.0 == 0x0303
    }
}
