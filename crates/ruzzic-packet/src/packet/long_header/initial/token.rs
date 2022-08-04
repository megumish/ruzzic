use ruzzic_common::var_int::VarInt;

use crate::packet::long_header::LongHeaderPacketTransformError;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(super) struct Token<'a> {
    pub value: &'a [u8],
    // Position just after reading token.
    pub position_after: usize,
}

pub(super) fn token(buf: &[u8], position: usize) -> Result<Token, LongHeaderPacketTransformError> {
    let (token_length_var_int, var_int_length) = VarInt::parse_and_get_raw_length(buf, position)?;
    let token_length = token_length_var_int.u64() as usize;
    let position = position + var_int_length;

    let value = buf
        .get(position..position + token_length)
        .ok_or(LongHeaderPacketTransformError::UnexpectedEnd(position))?;
    let position_after = position + token_length;
    Ok(Token {
        value,
        position_after,
    })
}

#[cfg(test)]
mod tests {
    use crate::packet::long_header::LongHeaderPacketTransformError;

    use super::{token, Token};

    #[test]
    fn token_length_8() -> Result<(), LongHeaderPacketTransformError> {
        let input = [&[8u8][..], &[2u8; 8]].concat();

        let token = token(&input, 0)?;

        assert_eq!(
            token,
            Token {
                value: &[2u8; 8],
                position_after: 1 // length of var int of token length
                 + 8 // length of token
            }
        );

        Ok(())
    }

    #[test]
    fn token_length_8_position_1() -> Result<(), LongHeaderPacketTransformError> {
        let input = [&[0u8][..], &[8u8][..], &[2u8; 8]].concat();

        let token = token(&input, 1)?;

        assert_eq!(
            token,
            Token {
                value: &[2u8; 8],
                position_after: 1 // length of padding
                 + 1 // length of var int of token length
                 + 8 // length of token
            }
        );

        Ok(())
    }
}
