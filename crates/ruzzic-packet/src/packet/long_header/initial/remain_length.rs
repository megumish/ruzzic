use ruzzic_common::var_int::VarInt;

use crate::packet::long_header::LongHeaderPacketTransformError;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RemainLength {
    pub value: usize,
    // Position just after reading value.
    pub position_after: usize,
}

pub fn remain_length(
    buf: &[u8],
    position: usize,
) -> Result<RemainLength, LongHeaderPacketTransformError> {
    let (remain_length_var_int, var_int_length) = VarInt::parse_and_get_raw_length(buf, position)?;
    let value = remain_length_var_int.u64() as usize;
    let position_after = position + var_int_length;
    Ok(RemainLength {
        value,
        position_after,
    })
}

#[cfg(test)]
mod tests {
    use crate::packet::long_header::LongHeaderPacketTransformError;

    use super::{remain_length, RemainLength};

    #[test]
    fn remain_length_8() -> Result<(), LongHeaderPacketTransformError> {
        let input = [8u8];

        let remain_length = remain_length(&input, 0)?;

        assert_eq!(
            remain_length,
            RemainLength {
                value: 8,
                position_after: 1 // length of var int of remain length
            }
        );

        Ok(())
    }

    #[test]
    fn remain_length_8_position_1() -> Result<(), LongHeaderPacketTransformError> {
        let input = [0u8, 8u8];

        let remain_length = remain_length(&input, 1)?;

        assert_eq!(
            remain_length,
            RemainLength {
                value: 8,
                position_after: 1 // length of padding
                 + 1 // length of var int of remain length
            }
        );

        Ok(())
    }
}
