use crate::packet::long_header::LongHeaderPacketTransformError;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Payload<'a> {
    pub value: &'a [u8],
    // Position just after reading value.
    pub position_after: usize,
}

pub fn payload(
    buf: &[u8],
    position: usize,
    length: usize,
) -> Result<Payload, LongHeaderPacketTransformError> {
    let value = buf
        .get(position..position + length)
        .ok_or(LongHeaderPacketTransformError::UnexpectedEnd(position))?;
    let position_after = position + length;
    Ok(Payload {
        value,
        position_after,
    })
}

#[cfg(test)]
mod tests {
    use crate::packet::long_header::LongHeaderPacketTransformError;

    use super::{payload, Payload};

    #[test]
    fn payload_length_8() -> Result<(), LongHeaderPacketTransformError> {
        let input = [2u8; 8];

        let payload = payload(&input, 0, 8)?;

        assert_eq!(
            payload,
            Payload {
                value: &[2u8; 8],
                position_after: 8 // length of payload
            }
        );

        Ok(())
    }

    #[test]
    fn payload_length_8_position_1() -> Result<(), LongHeaderPacketTransformError> {
        let input = [&[0u8][..], &[2u8; 8][..]].concat();

        let payload = payload(&input, 1, 8)?;

        assert_eq!(
            payload,
            Payload {
                value: &[2u8; 8],
                position_after: 1 // length of padding
                 + 8 // length of payload
            }
        );

        Ok(())
    }
}
