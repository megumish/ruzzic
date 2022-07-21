use ruzzic_common::EndpointType;

use crate::packet::PacketNumber;

#[derive(Debug, PartialEq)]
pub struct EndpointState {
    next_packet_number: PacketNumber,
    type_is: EndpointType,
}

impl EndpointState {
    pub(crate) fn new_server(packet_number: Option<PacketNumber>) -> Self {
        Self {
            next_packet_number: packet_number.unwrap_or(PacketNumber::zero()),
            type_is: EndpointType::Server,
        }
    }

    pub(crate) fn next_packet_number(&self) -> &PacketNumber {
        &self.next_packet_number
    }

    pub(crate) fn type_is(&self) -> &EndpointType {
        &self.type_is
    }
}
