use ruzzic_common::QuicVersion;

// unspecified long header packet
pub mod long_header;

#[derive(Debug, PartialEq, Clone)]
pub struct Packet<'a> {
    first_byte: u8,
    version: QuicVersion,
    next_bytes: &'a [u8],
}

impl<'a> Packet<'a> {
    pub(crate) fn first_byte(&self) -> u8 {
        self.first_byte
    }

    pub(crate) fn version(&self) -> QuicVersion {
        self.version.clone()
    }

    pub(crate) fn next_bytes<'b>(&'b self) -> &'b [u8] {
        self.next_bytes
    }

    #[cfg(test)]
    pub(crate) fn new(first_byte: u8, version: QuicVersion, next_bytes: &'a [u8]) -> Self {
        Self {
            first_byte,
            version,
            next_bytes,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Packet<'a> {
    type Error = PacketReadError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        let position = 0;

        let first_byte = *buf
            .get(position)
            .ok_or(Self::Error::UnexpectedEnd(position))?;
        let position = position + 1;

        let version_buf = buf
            .get(position..position + 4)
            .ok_or(Self::Error::UnexpectedEnd(position))?;
        let mut version = 0u32;
        for b in version_buf {
            version = (version << 8) + *b as u32;
        }
        let version = version.into();
        let position = position + 4;

        let next_bytes = &buf[position..];

        Ok(Self {
            first_byte,
            version,
            next_bytes,
        })
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PacketTransformError {
    #[error("not this kind")]
    NotThisKind(KindOfPacket),
    #[error("connection id is invalid length")]
    TransformToLong(#[from] TransformToLongHeaderError),
    #[error("unexpected end")]
    UnexpectedEnd(usize),
}

#[derive(Debug, PartialEq)]
pub enum KindOfPacket {
    LongHeader,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum TransformToLongHeaderError {
    #[error("connection id is invalid length")]
    ConnectionIdIsInvalidLength(usize),
}

#[derive(thiserror::Error, Debug)]
pub enum PacketReadError {
    #[error("unexpected end")]
    UnexpectedEnd(usize),
}

#[cfg(test)]
mod tests {
    use super::{Packet, PacketReadError};

    #[test]
    fn simple() -> Result<(), PacketReadError> {
        let input: &[u8] = &[
            0b0000_0000, // first byte
            0,
            0,
            0,
            0, // version
        ];

        let packet: Packet = input.try_into()?;

        assert_eq!(
            packet,
            Packet {
                first_byte: 0,
                version: 0u32.into(),
                next_bytes: &[],
            }
        );
        Ok(())
    }

    #[test]
    fn rfc9000_protected_client_initial_packet() -> Result<(), PacketReadError> {
        let input: &[u8] = &[
            192, 0, 0, 0, 1, 8, 131, 148, 200, 240, 62, 81, 87, 8, 0, 0, 68, 158, 123, 154, 236,
            52, 209, 177, 201, 141, 215, 104, 159, 184, 236, 17, 210, 66, 177, 35, 220, 155, 216,
            186, 185, 54, 180, 125, 146, 236, 53, 108, 11, 171, 125, 245, 151, 109, 39, 205, 68,
            159, 99, 48, 0, 153, 243, 153, 28, 38, 14, 196, 198, 13, 23, 179, 31, 132, 41, 21, 123,
            179, 90, 18, 130, 166, 67, 168, 210, 38, 44, 173, 103, 80, 12, 173, 184, 231, 55, 140,
            142, 183, 83, 158, 196, 212, 144, 95, 237, 27, 238, 31, 200, 170, 251, 161, 124, 117,
            14, 44, 122, 206, 1, 230, 0, 95, 128, 252, 183, 223, 98, 18, 48, 200, 55, 17, 179, 147,
            67, 250, 2, 140, 234, 127, 127, 181, 255, 137, 234, 194, 48, 130, 73, 160, 34, 82, 21,
            94, 35, 71, 182, 61, 88, 197, 69, 122, 253, 132, 208, 93, 255, 253, 178, 3, 146, 132,
            74, 232, 18, 21, 70, 130, 233, 207, 1, 47, 144, 33, 166, 240, 190, 23, 221, 208, 194,
            8, 77, 206, 37, 255, 155, 6, 205, 229, 53, 208, 249, 32, 162, 219, 27, 243, 98, 194,
            62, 89, 109, 17, 164, 245, 166, 207, 57, 72, 131, 138, 58, 236, 78, 21, 218, 248, 80,
            10, 110, 246, 158, 196, 227, 254, 182, 177, 217, 142, 97, 10, 200, 183, 236, 63, 175,
            106, 215, 96, 183, 186, 209, 219, 75, 163, 72, 94, 138, 148, 220, 37, 10, 227, 253,
            180, 30, 209, 95, 182, 168, 229, 235, 160, 252, 61, 214, 11, 200, 227, 12, 92, 66, 135,
            229, 56, 5, 219, 5, 154, 224, 100, 141, 178, 246, 66, 100, 237, 94, 57, 190, 46, 32,
            216, 45, 245, 102, 218, 141, 213, 153, 140, 202, 189, 174, 5, 48, 96, 174, 108, 123,
            67, 120, 232, 70, 210, 159, 55, 237, 123, 78, 169, 236, 93, 130, 231, 150, 27, 127, 37,
            169, 50, 56, 81, 246, 129, 213, 130, 54, 58, 165, 248, 153, 55, 245, 166, 114, 88, 191,
            99, 173, 111, 26, 11, 29, 150, 219, 212, 250, 221, 252, 239, 197, 38, 107, 166, 97, 23,
            34, 57, 92, 144, 101, 86, 190, 82, 175, 227, 245, 101, 99, 106, 209, 177, 125, 80, 139,
            115, 216, 116, 62, 235, 82, 75, 226, 43, 61, 203, 194, 199, 70, 141, 84, 17, 156, 116,
            104, 68, 154, 19, 216, 227, 185, 88, 17, 161, 152, 243, 73, 29, 227, 231, 254, 148, 43,
            51, 4, 7, 171, 248, 42, 78, 215, 193, 179, 17, 102, 58, 198, 152, 144, 244, 21, 112,
            21, 133, 61, 145, 233, 35, 3, 124, 34, 122, 51, 205, 213, 236, 40, 28, 163, 247, 156,
            68, 84, 107, 157, 144, 202, 0, 240, 100, 201, 158, 61, 217, 121, 17, 211, 159, 233,
            197, 208, 178, 58, 34, 154, 35, 76, 179, 97, 134, 196, 129, 158, 139, 156, 89, 39, 114,
            102, 50, 41, 29, 106, 65, 130, 17, 204, 41, 98, 226, 15, 228, 127, 235, 62, 223, 51,
            15, 44, 96, 58, 157, 72, 192, 252, 181, 105, 157, 191, 229, 137, 100, 37, 197, 186,
            196, 174, 232, 46, 87, 168, 90, 175, 78, 37, 19, 228, 240, 87, 150, 176, 123, 162, 238,
            71, 216, 5, 6, 248, 210, 194, 94, 80, 253, 20, 222, 113, 230, 196, 24, 85, 147, 2, 249,
            57, 176, 225, 171, 213, 118, 242, 121, 196, 178, 224, 254, 184, 92, 31, 40, 255, 24,
            245, 136, 145, 255, 239, 19, 46, 239, 47, 160, 147, 70, 174, 227, 60, 40, 235, 19, 15,
            242, 143, 91, 118, 105, 83, 51, 65, 19, 33, 25, 150, 210, 0, 17, 161, 152, 227, 252,
            67, 63, 159, 37, 65, 1, 10, 225, 124, 27, 242, 2, 88, 15, 96, 71, 71, 47, 179, 104, 87,
            254, 132, 59, 25, 245, 152, 64, 9, 221, 195, 36, 4, 78, 132, 122, 79, 74, 10, 179, 79,
            113, 149, 149, 222, 55, 37, 45, 98, 53, 54, 94, 155, 132, 57, 43, 6, 16, 133, 52, 157,
            115, 32, 58, 74, 19, 233, 111, 84, 50, 236, 15, 212, 161, 238, 101, 172, 205, 213, 227,
            144, 77, 245, 76, 29, 165, 16, 176, 255, 32, 220, 192, 199, 127, 203, 44, 14, 14, 182,
            5, 203, 5, 4, 219, 135, 99, 44, 243, 216, 180, 218, 230, 231, 5, 118, 157, 29, 227, 84,
            39, 1, 35, 203, 17, 69, 14, 252, 96, 172, 71, 104, 61, 123, 141, 15, 129, 19, 101, 86,
            95, 217, 140, 76, 142, 185, 54, 188, 171, 141, 6, 159, 195, 59, 216, 1, 176, 58, 222,
            162, 225, 251, 197, 170, 70, 61, 8, 202, 25, 137, 109, 43, 245, 154, 7, 27, 133, 30,
            108, 35, 144, 82, 23, 47, 41, 107, 251, 94, 114, 64, 71, 144, 162, 24, 16, 20, 243,
            185, 74, 78, 151, 209, 23, 180, 56, 19, 3, 104, 204, 57, 219, 178, 209, 152, 6, 90,
            227, 152, 101, 71, 146, 108, 210, 22, 47, 64, 162, 159, 12, 60, 135, 69, 192, 245, 15,
            186, 56, 82, 229, 102, 212, 69, 117, 194, 157, 57, 160, 63, 12, 218, 114, 25, 132, 182,
            244, 64, 89, 31, 53, 94, 18, 212, 57, 255, 21, 10, 171, 118, 19, 73, 157, 189, 73, 173,
            171, 200, 103, 110, 239, 2, 59, 21, 182, 91, 252, 92, 160, 105, 72, 16, 159, 35, 243,
            80, 219, 130, 18, 53, 53, 235, 138, 116, 51, 189, 171, 203, 144, 146, 113, 166, 236,
            188, 181, 139, 147, 106, 136, 205, 78, 143, 46, 111, 245, 128, 1, 117, 241, 19, 37, 61,
            143, 169, 202, 136, 133, 194, 245, 82, 230, 87, 220, 96, 63, 37, 46, 26, 142, 48, 143,
            118, 240, 190, 121, 226, 251, 143, 93, 95, 187, 226, 227, 14, 202, 221, 34, 7, 35, 200,
            192, 174, 168, 7, 140, 223, 203, 56, 104, 38, 63, 248, 240, 148, 0, 84, 218, 72, 120,
            24, 147, 167, 228, 154, 213, 175, 244, 175, 48, 12, 216, 4, 166, 182, 39, 154, 179,
            255, 58, 251, 100, 73, 28, 133, 25, 74, 171, 118, 13, 88, 166, 6, 101, 79, 159, 68, 0,
            232, 179, 133, 145, 53, 111, 191, 100, 37, 172, 162, 109, 200, 82, 68, 37, 159, 242,
            177, 156, 65, 185, 249, 111, 60, 169, 236, 29, 222, 67, 77, 167, 210, 211, 146, 185, 5,
            221, 243, 209, 249, 175, 147, 209, 175, 89, 80, 189, 73, 63, 90, 167, 49, 180, 5, 109,
            243, 27, 210, 103, 182, 185, 10, 7, 152, 49, 170, 245, 121, 190, 10, 57, 1, 49, 55,
            170, 198, 212, 4, 245, 24, 207, 212, 104, 64, 100, 126, 120, 191, 231, 6, 202, 76, 245,
            233, 197, 69, 62, 159, 124, 253, 43, 139, 76, 141, 22, 154, 68, 229, 92, 136, 212, 169,
            167, 249, 71, 66, 65, 226, 33, 175, 68, 134, 0, 24, 171, 8, 86, 151, 46, 25, 76, 217,
            52,
        ];

        let packet: Packet = input.try_into()?;

        assert_eq!(
            packet,
            Packet {
                first_byte: 192,
                version: ruzzic_common::QuicVersion::Rfc9000,
                next_bytes: &[
                    8, 131, 148, 200, 240, 62, 81, 87, 8, 0, 0, 68, 158, 123, 154, 236, 52, 209,
                    177, 201, 141, 215, 104, 159, 184, 236, 17, 210, 66, 177, 35, 220, 155, 216,
                    186, 185, 54, 180, 125, 146, 236, 53, 108, 11, 171, 125, 245, 151, 109, 39,
                    205, 68, 159, 99, 48, 0, 153, 243, 153, 28, 38, 14, 196, 198, 13, 23, 179, 31,
                    132, 41, 21, 123, 179, 90, 18, 130, 166, 67, 168, 210, 38, 44, 173, 103, 80,
                    12, 173, 184, 231, 55, 140, 142, 183, 83, 158, 196, 212, 144, 95, 237, 27, 238,
                    31, 200, 170, 251, 161, 124, 117, 14, 44, 122, 206, 1, 230, 0, 95, 128, 252,
                    183, 223, 98, 18, 48, 200, 55, 17, 179, 147, 67, 250, 2, 140, 234, 127, 127,
                    181, 255, 137, 234, 194, 48, 130, 73, 160, 34, 82, 21, 94, 35, 71, 182, 61, 88,
                    197, 69, 122, 253, 132, 208, 93, 255, 253, 178, 3, 146, 132, 74, 232, 18, 21,
                    70, 130, 233, 207, 1, 47, 144, 33, 166, 240, 190, 23, 221, 208, 194, 8, 77,
                    206, 37, 255, 155, 6, 205, 229, 53, 208, 249, 32, 162, 219, 27, 243, 98, 194,
                    62, 89, 109, 17, 164, 245, 166, 207, 57, 72, 131, 138, 58, 236, 78, 21, 218,
                    248, 80, 10, 110, 246, 158, 196, 227, 254, 182, 177, 217, 142, 97, 10, 200,
                    183, 236, 63, 175, 106, 215, 96, 183, 186, 209, 219, 75, 163, 72, 94, 138, 148,
                    220, 37, 10, 227, 253, 180, 30, 209, 95, 182, 168, 229, 235, 160, 252, 61, 214,
                    11, 200, 227, 12, 92, 66, 135, 229, 56, 5, 219, 5, 154, 224, 100, 141, 178,
                    246, 66, 100, 237, 94, 57, 190, 46, 32, 216, 45, 245, 102, 218, 141, 213, 153,
                    140, 202, 189, 174, 5, 48, 96, 174, 108, 123, 67, 120, 232, 70, 210, 159, 55,
                    237, 123, 78, 169, 236, 93, 130, 231, 150, 27, 127, 37, 169, 50, 56, 81, 246,
                    129, 213, 130, 54, 58, 165, 248, 153, 55, 245, 166, 114, 88, 191, 99, 173, 111,
                    26, 11, 29, 150, 219, 212, 250, 221, 252, 239, 197, 38, 107, 166, 97, 23, 34,
                    57, 92, 144, 101, 86, 190, 82, 175, 227, 245, 101, 99, 106, 209, 177, 125, 80,
                    139, 115, 216, 116, 62, 235, 82, 75, 226, 43, 61, 203, 194, 199, 70, 141, 84,
                    17, 156, 116, 104, 68, 154, 19, 216, 227, 185, 88, 17, 161, 152, 243, 73, 29,
                    227, 231, 254, 148, 43, 51, 4, 7, 171, 248, 42, 78, 215, 193, 179, 17, 102, 58,
                    198, 152, 144, 244, 21, 112, 21, 133, 61, 145, 233, 35, 3, 124, 34, 122, 51,
                    205, 213, 236, 40, 28, 163, 247, 156, 68, 84, 107, 157, 144, 202, 0, 240, 100,
                    201, 158, 61, 217, 121, 17, 211, 159, 233, 197, 208, 178, 58, 34, 154, 35, 76,
                    179, 97, 134, 196, 129, 158, 139, 156, 89, 39, 114, 102, 50, 41, 29, 106, 65,
                    130, 17, 204, 41, 98, 226, 15, 228, 127, 235, 62, 223, 51, 15, 44, 96, 58, 157,
                    72, 192, 252, 181, 105, 157, 191, 229, 137, 100, 37, 197, 186, 196, 174, 232,
                    46, 87, 168, 90, 175, 78, 37, 19, 228, 240, 87, 150, 176, 123, 162, 238, 71,
                    216, 5, 6, 248, 210, 194, 94, 80, 253, 20, 222, 113, 230, 196, 24, 85, 147, 2,
                    249, 57, 176, 225, 171, 213, 118, 242, 121, 196, 178, 224, 254, 184, 92, 31,
                    40, 255, 24, 245, 136, 145, 255, 239, 19, 46, 239, 47, 160, 147, 70, 174, 227,
                    60, 40, 235, 19, 15, 242, 143, 91, 118, 105, 83, 51, 65, 19, 33, 25, 150, 210,
                    0, 17, 161, 152, 227, 252, 67, 63, 159, 37, 65, 1, 10, 225, 124, 27, 242, 2,
                    88, 15, 96, 71, 71, 47, 179, 104, 87, 254, 132, 59, 25, 245, 152, 64, 9, 221,
                    195, 36, 4, 78, 132, 122, 79, 74, 10, 179, 79, 113, 149, 149, 222, 55, 37, 45,
                    98, 53, 54, 94, 155, 132, 57, 43, 6, 16, 133, 52, 157, 115, 32, 58, 74, 19,
                    233, 111, 84, 50, 236, 15, 212, 161, 238, 101, 172, 205, 213, 227, 144, 77,
                    245, 76, 29, 165, 16, 176, 255, 32, 220, 192, 199, 127, 203, 44, 14, 14, 182,
                    5, 203, 5, 4, 219, 135, 99, 44, 243, 216, 180, 218, 230, 231, 5, 118, 157, 29,
                    227, 84, 39, 1, 35, 203, 17, 69, 14, 252, 96, 172, 71, 104, 61, 123, 141, 15,
                    129, 19, 101, 86, 95, 217, 140, 76, 142, 185, 54, 188, 171, 141, 6, 159, 195,
                    59, 216, 1, 176, 58, 222, 162, 225, 251, 197, 170, 70, 61, 8, 202, 25, 137,
                    109, 43, 245, 154, 7, 27, 133, 30, 108, 35, 144, 82, 23, 47, 41, 107, 251, 94,
                    114, 64, 71, 144, 162, 24, 16, 20, 243, 185, 74, 78, 151, 209, 23, 180, 56, 19,
                    3, 104, 204, 57, 219, 178, 209, 152, 6, 90, 227, 152, 101, 71, 146, 108, 210,
                    22, 47, 64, 162, 159, 12, 60, 135, 69, 192, 245, 15, 186, 56, 82, 229, 102,
                    212, 69, 117, 194, 157, 57, 160, 63, 12, 218, 114, 25, 132, 182, 244, 64, 89,
                    31, 53, 94, 18, 212, 57, 255, 21, 10, 171, 118, 19, 73, 157, 189, 73, 173, 171,
                    200, 103, 110, 239, 2, 59, 21, 182, 91, 252, 92, 160, 105, 72, 16, 159, 35,
                    243, 80, 219, 130, 18, 53, 53, 235, 138, 116, 51, 189, 171, 203, 144, 146, 113,
                    166, 236, 188, 181, 139, 147, 106, 136, 205, 78, 143, 46, 111, 245, 128, 1,
                    117, 241, 19, 37, 61, 143, 169, 202, 136, 133, 194, 245, 82, 230, 87, 220, 96,
                    63, 37, 46, 26, 142, 48, 143, 118, 240, 190, 121, 226, 251, 143, 93, 95, 187,
                    226, 227, 14, 202, 221, 34, 7, 35, 200, 192, 174, 168, 7, 140, 223, 203, 56,
                    104, 38, 63, 248, 240, 148, 0, 84, 218, 72, 120, 24, 147, 167, 228, 154, 213,
                    175, 244, 175, 48, 12, 216, 4, 166, 182, 39, 154, 179, 255, 58, 251, 100, 73,
                    28, 133, 25, 74, 171, 118, 13, 88, 166, 6, 101, 79, 159, 68, 0, 232, 179, 133,
                    145, 53, 111, 191, 100, 37, 172, 162, 109, 200, 82, 68, 37, 159, 242, 177, 156,
                    65, 185, 249, 111, 60, 169, 236, 29, 222, 67, 77, 167, 210, 211, 146, 185, 5,
                    221, 243, 209, 249, 175, 147, 209, 175, 89, 80, 189, 73, 63, 90, 167, 49, 180,
                    5, 109, 243, 27, 210, 103, 182, 185, 10, 7, 152, 49, 170, 245, 121, 190, 10,
                    57, 1, 49, 55, 170, 198, 212, 4, 245, 24, 207, 212, 104, 64, 100, 126, 120,
                    191, 231, 6, 202, 76, 245, 233, 197, 69, 62, 159, 124, 253, 43, 139, 76, 141,
                    22, 154, 68, 229, 92, 136, 212, 169, 167, 249, 71, 66, 65, 226, 33, 175, 68,
                    134, 0, 24, 171, 8, 86, 151, 46, 25, 76, 217, 52
                ],
            }
        );
        Ok(())
    }
}
