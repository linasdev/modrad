use crate::eap::code::EapPacketCode;
use crate::eap::data::EapPacketData;
use crate::radius::metadata::RadiusPacketMetadata;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use crate::identifier::EapIdentifier;

pub mod code;
pub mod data;

const EAP_PACKET_HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub enum EapPacketError {
    NotEnoughData,
    InvalidCode,
}

pub struct EapPacket {
    identifier: EapIdentifier,
    data: EapPacketData,
}

impl EapPacket {
    pub fn new(identifier: EapIdentifier, data: EapPacketData) -> Self {
        Self { identifier, data }
    }

    pub fn code(&self) -> EapPacketCode {
        self.data.code()
    }

    pub fn identifier(&self) -> EapIdentifier {
        self.identifier
    }

    pub fn length(&self) -> usize {
        EAP_PACKET_HEADER_SIZE + self.data.length()
    }

    pub fn data(&self) -> &EapPacketData {
        &self.data
    }
}

impl From<EapPacket> for Vec<u8> {
    fn from(packet: EapPacket) -> Self {
        let mut buffer = Vec::with_capacity(packet.length());

        buffer.push(packet.code().into()); // byte 0
        buffer.push(packet.identifier.into()); // byte 1

        for byte in u16::to_be_bytes(packet.length() as u16) {
            buffer.push(byte); // bytes 2 & 3
        }

        buffer.extend_from_slice(&Vec::from(packet.data)); // bytes 4 - length

        buffer
    }
}

impl TryFrom<&[u8]> for EapPacket {
    type Error = EapPacketError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < EAP_PACKET_HEADER_SIZE {
            return Err(EapPacketError::NotEnoughData);
        }

        let code = EapPacketCode::try_from(buffer[0]).map_err(|_| EapPacketError::InvalidCode)?;
        let identifier = buffer[1].into();
        let length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;

        if buffer.len() < length {
            return Err(EapPacketError::NotEnoughData);
        }

        let data = EapPacketData::try_from((code, &buffer[EAP_PACKET_HEADER_SIZE..length]))?;

        Ok(Self { identifier, data })
    }
}

impl RadiusPacketMetadata for EapPacket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Debug for EapPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EapPacket")
            .field("identifier", &self.identifier())
            .field("length", &self.length())
            .field("data", self.data())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eap::data::EapPacketTypeData;
    use googletest::prelude::*;

    #[test]
    fn should_convert_from_eap_packet_request_to_byte_buffer() {
        let packet = EapPacket::new(
            0.into(),
            EapPacketData::Request {
                type_data: EapPacketTypeData::Identity(vec![1, 2, 3]),
            },
        );
        let result = Vec::from(packet);

        let expected_result = vec![
            1, // Request
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            1, // Identity
            1, 2, 3, // Identity data
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_response_to_byte_buffer() {
        let packet = EapPacket::new(
            0.into(),
            EapPacketData::Response {
                type_data: EapPacketTypeData::Identity(vec![1, 2, 3]),
            },
        );
        let result = Vec::from(packet);

        let expected_result = vec![
            2, // Response
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            1, // Identity
            1, 2, 3, // Identity data
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_success_to_byte_buffer() {
        let packet = EapPacket::new(0.into(), EapPacketData::Success);
        let result = Vec::from(packet);

        let expected_result = vec![
            3, // Success
            0, // Identifier
            0, // Length MSB
            4, // Length LSB
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_failure_to_byte_buffer() {
        let packet = EapPacket::new(0.into(), EapPacketData::Failure);
        let result = Vec::from(packet);

        let expected_result = vec![
            4, // Failure
            0, // Identifier
            0, // Length MSB
            4, // Length LSB
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_request_other_to_byte_buffer() {
        let packet = EapPacket::new(
            0.into(),
            EapPacketData::Request {
                type_data: EapPacketTypeData::Other(0, vec![1, 2, 3]),
            },
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            1, // Request
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            0, // Type
            1, 2, 3, // Type data
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_byte_buffer_to_eap_packet_request() {
        let buffer = vec![
            1, // Request
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            1, // Identity
            1, 2, 3, // Identity data
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacket {
                identifier: eq(&0.into()),
                data: matches_pattern!(EapPacketData::Request {
                    type_data: matches_pattern!(EapPacketTypeData::Identity(&[1, 2, 3])),
                }),
            },)
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_eap_packet_response() {
        let buffer = vec![
            2, // Response
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            1, // Identity
            1, 2, 3, // Identity data
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacket {
                identifier: eq(&0.into()),
                data: matches_pattern!(EapPacketData::Response {
                    type_data: matches_pattern!(EapPacketTypeData::Identity(&[1, 2, 3])),
                }),
            },)
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_eap_packet_success() {
        let buffer = vec![
            3, // Success
            0, // Identifier
            0, // Length MSB
            4, // Length LSB
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacket {
                identifier: eq(&0.into()),
                data: matches_pattern!(EapPacketData::Success),
            },)
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_eap_packet_failure() {
        let buffer = vec![
            4, // Failure
            0, // Identifier
            0, // Length MSB
            4, // Length LSB
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacket {
                identifier: eq(&0.into()),
                data: matches_pattern!(EapPacketData::Failure),
            },)
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_eap_packet_request_other() {
        let buffer = vec![
            1, // Request
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            0, // Type
            1, 2, 3, // Type data
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacket {
                identifier: eq(&0.into()),
                data: matches_pattern!(EapPacketData::Request {
                    type_data: matches_pattern!(EapPacketTypeData::Other(eq(&0), eq(&[1, 2, 3]))),
                }),
            },)
        );
    }

    #[test]
    fn should_not_convert_from_byte_buffer_to_eap_packet_request_when_there_is_not_enough_data() {
        let buffer = vec![
            1, // Request
            0, // Identifier
            0, // Length MSB
            8, // Length LSB
            1, // Identity
            1, 2, // Missing byte
        ];

        let result = EapPacket::try_from(&buffer[..]).unwrap_err();

        assert_that!(result, matches_pattern!(EapPacketError::NotEnoughData));
    }
}
