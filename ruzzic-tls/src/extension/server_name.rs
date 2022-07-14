use std::io::{Cursor, Read};

use byteorder::ReadBytesExt;
use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    names: Vec<ServerName>,
    total_length: usize,
}

#[derive(Debug)]
pub struct ServerName {
    name_type: NameType,
    host_name: Option<Vec<u8>>,
}

#[derive(Debug)]
enum NameType {
    HostName,
    Unknown(u8),
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<byteorder::NetworkEndian>()? as usize;
        let mut input = {
            let mut buf = vec![0u8; length as usize];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };
        let mut names = Vec::new();
        let mut total_length = 0usize;
        while total_length < length {
            let _ = input.read_u16::<byteorder::NetworkEndian>()? as usize;
            let name_type = match input.read_u8()? {
                0 => NameType::HostName,
                x => NameType::Unknown(x),
            };
            let host_name = match name_type {
                NameType::HostName => {
                    let length = input.read_u16::<byteorder::NetworkEndian>()? as usize;
                    let mut host_name = vec![0u8; length as usize];
                    input.read_exact(&mut host_name)?;
                    Some(host_name)
                }
                NameType::Unknown(_) => None,
            };
            total_length += 2 + 1 + 2 + host_name.as_ref().map(|x| x.len()).unwrap_or(0);
            names.push(ServerName {
                name_type,
                host_name,
            });
        }
        Ok(Self {
            names,
            total_length,
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.total_length
    }
}
