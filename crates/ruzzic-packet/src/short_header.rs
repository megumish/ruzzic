pub struct OneRttPacket<'a> {
    first_byte: u8,
    destination_connection_id: &'a [u8],
    packet_number: u32,
    packet_payload: &'a [u8],
}
