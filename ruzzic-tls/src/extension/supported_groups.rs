use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    named_curve_list: Vec<NamedCurve>,
    total_length: usize,
}

#[derive(Debug)]
pub enum NamedCurve {
    Deprecated(u16),
    Reserved(u16),
    Secp256r1,
    Secp384rl,
    Secp521r1,
    X25519,
    X448,
    Others(u16),
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()? as usize;
        let mut total_length = 0usize;
        let mut named_curve_list = Vec::new();
        while total_length < length {
            let named_curve = match input.read_u16::<NetworkEndian>()? {
                x @ 0x01..=0x16 => NamedCurve::Deprecated(x),
                x @ 0xFF01..=0xFF02 => NamedCurve::Deprecated(x),
                x @ 0xFE00..=0xFEFF => NamedCurve::Reserved(x),
                0x17 => NamedCurve::Secp256r1,
                0x18 => NamedCurve::Secp384rl,
                0x19 => NamedCurve::Secp521r1,
                0x1D => NamedCurve::X25519,
                0x1E => NamedCurve::X448,
                x => NamedCurve::Others(x),
            };
            named_curve_list.push(named_curve);
            total_length += 2;
        }
        println!("{named_curve_list:?}");

        Ok(Self {
            named_curve_list,
            total_length,
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        todo!()
    }
}
