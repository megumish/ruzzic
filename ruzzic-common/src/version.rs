#[derive(Debug, Clone, PartialEq)]
pub enum QuicVersion {
    Rfc9000,
    Others(u32),
}

pub type QuicVersions = Vec<QuicVersion>;

impl QuicVersion {
    pub fn initial_salt(&self) -> [u8; 0x14] {
        match self {
            QuicVersion::Rfc9000 => [
                0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8,
                0x0c, 0xad, 0xcc, 0xbb, 0x7f, 0x0a,
            ],
            QuicVersion::Others(_) => unimplemented!(),
        }
    }
}
