#[derive(Debug, PartialEq)]
pub struct StreamID(pub(crate) u64);

#[derive(Debug, PartialEq)]
pub struct StreamData(pub(crate) Vec<u8>);
