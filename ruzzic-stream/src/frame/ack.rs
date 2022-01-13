use crate::{packet::PacketNumber, read_varint, VarInt};
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct Body {
    // 通常は相手がAckフレームを作る前に受信した最大のPacketNumberらしい
    largest_acknowledged: PacketNumber,
    ack_delay: VarInt,
    ack_ranges: Vec<AckRange>,
    ecn_counts: Option<ECNCounts>,
}

#[derive(Debug, PartialEq)]
pub struct AckRange {
    gap: VarInt,
    length: VarInt,
}

#[derive(Debug, PartialEq)]
pub struct ECNCounts {
    ect0_count: VarInt,
    ect1_count: VarInt,
    ecn_ce_count: VarInt,
}

impl Body {
    pub fn read_bytes(input: &mut impl Read, frame_type: u64) -> Result<Self, std::io::Error> {
        let largest_acknowledged = read_varint(input)?;
        let ack_delay = read_varint(input)?;
        let ack_ranges_length = read_varint(input)?;
        let mut ack_ranges = Vec::new();
        for _ in 0..ack_ranges_length.to_u64() {
            let gap = read_varint(input)?;
            let length = read_varint(input)?;
            ack_ranges.push(AckRange { gap, length });
        }
        let ecn_counts = if frame_type == 0x03 {
            let ect0_count = read_varint(input)?;
            let ect1_count = read_varint(input)?;
            let ecn_ce_count = read_varint(input)?;
            Some(ECNCounts {
                ect0_count,
                ect1_count,
                ecn_ce_count,
            })
        } else {
            None
        };
        Ok(Self {
            largest_acknowledged: PacketNumber(largest_acknowledged.to_u64() as u32),
            ack_delay,
            ack_ranges,
            ecn_counts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn ack_frame_0x02() {
        let buf = [0, 0, 1, 0, 0];
        let mut input = Cursor::new(buf);
        let actual = Body::read_bytes(&mut input, 0x02).unwrap();
        let expect = Body {
            largest_acknowledged: PacketNumber(0x00),
            ack_delay: VarInt(0),
            ack_ranges: vec![AckRange {
                gap: VarInt(0),
                length: VarInt(0),
            }],
            ecn_counts: None,
        };
        assert_eq!(actual, expect);
    }

    #[test]
    fn ack_frame_0x03() {
        let buf = [0, 0, 1, 0, 0, 0, 0, 0];
        let mut input = Cursor::new(buf);
        let actual = Body::read_bytes(&mut input, 0x03).unwrap();
        let expect = Body {
            largest_acknowledged: PacketNumber(0x00),
            ack_delay: VarInt(0),
            ack_ranges: vec![AckRange {
                gap: VarInt(0),
                length: VarInt(0),
            }],
            ecn_counts: Some(ECNCounts {
                ect0_count: VarInt(0),
                ect1_count: VarInt(0),
                ecn_ce_count: VarInt(0),
            }),
        };
        assert_eq!(actual, expect);
    }
}
