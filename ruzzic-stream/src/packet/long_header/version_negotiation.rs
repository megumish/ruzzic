use super::{ConnectionIDPair, Versions};
use crate::{read_bytes_to::FromReadBytesWith, ReadBytesTo};

#[derive(Debug, PartialEq)]
pub struct Body {
    pub connection_id_pair: ConnectionIDPair,
    pub supported_versions: Versions,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let connection_id_pair = input.read_bytes_to()?;
        let supported_versions = input.read_bytes_to()?;
        Ok(Self {
            connection_id_pair,
            supported_versions,
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::{read_bytes_to::ReadBytesTo, Version};

    use super::*;
    use byteorder::{BigEndian, ByteOrder};
    use std::io::Cursor;

    #[test]
    fn version_negotiation_packet() {
        let buf = {
            let destination_id = [0x01];
            let destination_id_length = [destination_id.len() as u8];

            let source_id = [0x02, 0x11];
            let source_id_length = [source_id.len() as u8];

            let versions = [0x01, 0x02]
                .iter()
                .map(|version| {
                    let mut buf = [0u8; 4];
                    BigEndian::write_u32(&mut buf, *version);
                    buf
                })
                .collect::<Vec<_>>()
                .concat();

            [
                &destination_id_length[..],
                &destination_id[..],
                &source_id_length[..],
                &source_id[..],
                &versions[..],
            ]
            .concat()
        };
        let mut input = Cursor::new(buf);

        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            connection_id_pair: ConnectionIDPair {
                destination_id: vec![0x01],
                source_id: vec![0x02, 0x11],
            },
            supported_versions: Versions(vec![Version(0x01), Version(0x02)]),
        };
        assert_eq!(actual, expected);
    }
}
