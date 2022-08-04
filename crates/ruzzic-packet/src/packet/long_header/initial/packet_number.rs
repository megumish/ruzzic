use crate::packet::long_header::LongHeaderPacketTransformError;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct PacketNumber {
    pub value: u32,
    // Position just after reading value.
    pub position_after: usize,
}

pub fn packet_number(
    buf: &[u8],
    position: usize,
    length: usize,
) -> Result<PacketNumber, LongHeaderPacketTransformError> {
    let value = {
        let buf = buf
            .get(position..position + length)
            .ok_or(LongHeaderPacketTransformError::UnexpectedEnd(position))?;
        let mut packet_number = 0u32;
        for b in buf {
            packet_number = (packet_number << 8) + *b as u32;
        }
        packet_number
    };
    let position_after = position + length;
    Ok(PacketNumber {
        value,
        position_after,
    })
}

#[cfg(test)]
mod tests {
    use crate::packet::long_header::LongHeaderPacketTransformError;

    use super::{packet_number, PacketNumber};

    #[test]
    fn packet_number_length_4() -> Result<(), LongHeaderPacketTransformError> {
        let input = [2u8; 4];

        let packet_number = packet_number(&input, 0, 4)?;

        assert_eq!(
            packet_number,
            PacketNumber {
                value: 0x02020202,
                position_after: 4 // length of packet number
            }
        );

        Ok(())
    }

    #[test]
    fn packet_number_length_4_position_1() -> Result<(), LongHeaderPacketTransformError> {
        let input = [&[0u8][..], &[2u8; 4][..]].concat();

        let remain_length = packet_number(&input, 1, 4)?;

        assert_eq!(
            remain_length,
            PacketNumber {
                value: 0x02020202,
                position_after: 1 // length of padding
                 + 4 // length of packet number
            }
        );

        Ok(())
    }
}
