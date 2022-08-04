use ruzzic_common::QuicVersions;

use crate::packet::long_header::LongHeaderPacketTransformError;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct SupportedVersions {
    pub value: QuicVersions,
    // Position just after reading value.
    pub position_after: usize,
}

pub fn supported_versions(
    buf: &[u8],
    position: usize,
) -> Result<SupportedVersions, LongHeaderPacketTransformError> {
    if buf.len() % 4 != 0 {
        return Err(LongHeaderPacketTransformError::UnexpectedEnd(
            buf.len() - (buf.len() % 4),
        ));
    }
    let supported_versions_length = buf.len() / 4;
    let mut value = Vec::new();
    for i in 0..supported_versions_length {
        let supported_version_bytes = buf.get(position + i * 4..position + (i + 1) * 4).ok_or(
            LongHeaderPacketTransformError::UnexpectedEnd(position + i * 4),
        )?;
        let mut version = 0u32;
        for b in supported_version_bytes {
            version = (version << 1) + *b as u32;
        }
        value.push(version.into());
    }
    let position_after = position + supported_versions_length * 4;
    Ok(SupportedVersions {
        value,
        position_after,
    })
}

#[cfg(test)]
mod tests {
    use ruzzic_common::QuicVersion;

    use crate::packet::long_header::LongHeaderPacketTransformError;

    use super::{supported_versions, SupportedVersions};

    #[test]
    fn supported_versions_length_8() -> Result<(), LongHeaderPacketTransformError> {
        let input: Vec<u8> = [&[0, 0, 0, 1][..], &[0, 0, 0, 2][..]].concat();

        let supported_versions = supported_versions(&input, 0)?;

        assert_eq!(
            supported_versions,
            SupportedVersions {
                value: vec![QuicVersion::Rfc9000, QuicVersion::Others(2)],
                position_after: 8 // length of packet number
            }
        );

        Ok(())
    }

    #[test]
    fn supported_versions_length_9() -> Result<(), LongHeaderPacketTransformError> {
        let input: Vec<u8> = [&[0, 0, 0, 1][..], &[0, 0, 0, 2][..], &[0][..]].concat();

        let result = supported_versions(&input, 0);

        assert_eq!(
            result,
            Err(LongHeaderPacketTransformError::UnexpectedEnd(8))
        );

        Ok(())
    }
}
