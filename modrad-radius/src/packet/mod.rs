use crate::packet::attribute::RadiusPacketAttribute;
use crate::packet::code::RadiusPacketCode;
use crate::tag_length_value::TagLengthValueError;
use std::fmt::{Debug, Formatter};

pub mod attribute;
pub mod code;

const RADIUS_PACKET_HEADER_SIZE: usize = 20;

#[derive(Debug)]
pub enum RadiusPacketError {
    NotEnoughData,
    TagLengthValue(TagLengthValueError),
}

pub struct RadiusPacket {
    code: RadiusPacketCode,
    identifier: u8,
    authenticator: [u8; 16],
    attributes: Vec<RadiusPacketAttribute>,
}

impl RadiusPacket {
    pub fn new(
        code: RadiusPacketCode,
        identifier: u8,
        authenticator: [u8; 16],
        attributes: Vec<RadiusPacketAttribute>,
    ) -> Self {
        Self {
            code,
            identifier,
            authenticator,
            attributes,
        }
    }

    pub fn code(&self) -> RadiusPacketCode {
        self.code
    }

    pub fn identifier(&self) -> u8 {
        self.identifier
    }

    pub fn length(&self) -> usize {
        RADIUS_PACKET_HEADER_SIZE
            + self
            .attributes
            .iter()
            .map(RadiusPacketAttribute::length)
            .sum::<usize>()
    }

    pub fn authenticator(&self) -> &[u8; 16] {
        &self.authenticator
    }

    pub fn attributes(&self) -> &[RadiusPacketAttribute] {
        &self.attributes
    }
}

impl From<RadiusPacket> for Vec<u8> {
    fn from(packet: RadiusPacket) -> Self {
        let mut buffer = Vec::with_capacity(packet.length());

        buffer.push(packet.code.into()); // byte 0
        buffer.push(packet.identifier); // byte 1

        for byte in u16::to_be_bytes(packet.length() as u16) {
            buffer.push(byte); // bytes 2 & 3
        }

        for byte in packet.authenticator.into_iter() {
            buffer.push(byte); // bytes 4 - 19
        }

        for attribute in packet.attributes.into_iter() {
            buffer.extend_from_slice(&Vec::from(attribute)); // bytes 20 - length
        }

        buffer
    }
}

impl TryFrom<&[u8]> for RadiusPacket {
    type Error = RadiusPacketError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < RADIUS_PACKET_HEADER_SIZE {
            return Err(RadiusPacketError::NotEnoughData);
        }

        let code = RadiusPacketCode::from(buffer[0]);
        let identifier = buffer[1];

        let length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;

        if buffer.len() < length {
            return Err(RadiusPacketError::NotEnoughData);
        }

        let authenticator: [u8; 16] = buffer[4..RADIUS_PACKET_HEADER_SIZE].try_into().unwrap();

        let mut offset = RADIUS_PACKET_HEADER_SIZE;
        let mut attributes = vec![];
        while offset < length {
            let attribute = RadiusPacketAttribute::try_from(&buffer[offset..])?;
            offset += attribute.length();
            attributes.push(attribute);
        }

        Ok(Self {
            code,
            identifier,
            authenticator,
            attributes,
        })
    }
}

impl Debug for RadiusPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadiusPacket")
            .field("code", &self.code)
            .field("identifier", &self.identifier)
            .field("length", &self.length())
            .field("authenticator", &self.authenticator)
            .field("attributes", &self.attributes)
            .finish()
    }
}

impl From<TagLengthValueError> for RadiusPacketError {
    fn from(error: TagLengthValueError) -> Self {
        RadiusPacketError::TagLengthValue(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::attribute::RadiusPacketAttributeType;
    use googletest::prelude::*;

    #[test]
    fn should_convert_from_radius_packet_without_attributes_to_byte_buffer() {
        let packet = RadiusPacket::new(
            RadiusPacketCode::AccessRequest,
            1,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            vec![],
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            20, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_radius_packet_with_single_attribute_to_byte_buffer() {
        let packet = RadiusPacket::new(
            RadiusPacketCode::AccessRequest,
            1,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            vec![RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::UserName,
                vec![],
            )],
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            22, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
            1,  // User-Name
            2,  // Length
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_radius_packet_with_multiple_attributes_to_byte_buffer() {
        let packet = RadiusPacket::new(
            RadiusPacketCode::AccessRequest,
            1,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            vec![
                RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::UserName,
                    b"hello".to_vec(),
                ),
                RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::UserPassword,
                    b"world".to_vec(),
                ),
            ],
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            34, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
            1,  // User-Name
            7,  // Length
            b'h', b'e', b'l', b'l', b'o', // Value
            2,    // User-Password
            7,    // Length
            b'w', b'o', b'r', b'l', b'd', // Value
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_byte_buffer_to_radius_packet_without_attributes() {
        let buffer = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            20, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
        ];

        let result = RadiusPacket::try_from(&buffer[..]).unwrap();

        assert_that!(result.code(), eq(RadiusPacketCode::AccessRequest));
        assert_that!(result.identifier(), eq(1));
        assert_that!(result.length(), eq(20));
        assert_that!(
            result.authenticator(),
            eq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        );
        assert_that!(result.attributes(), is_empty());
    }

    #[test]
    fn should_convert_from_byte_buffer_to_radius_packet_with_single_attribute() {
        let buffer = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            22, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
            1,  // User-Name
            2,  // Length
        ];

        let result = RadiusPacket::try_from(&buffer[..]).unwrap();

        assert_that!(result.code(), eq(RadiusPacketCode::AccessRequest));
        assert_that!(result.identifier(), eq(1));
        assert_that!(result.length(), eq(22));
        assert_that!(
            result.authenticator(),
            eq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        );
        assert_that!(
            result.attributes(),
            elements_are![eq(&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::UserName,
                vec![]
            )),]
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_radius_packet_with_multiple_attributes() {
        let buffer = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            34, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
            1,  // User-Name
            7,  // Length
            b'h', b'e', b'l', b'l', b'o', // Value
            2,    // User-Password
            7,    // Length
            b'w', b'o', b'r', b'l', b'd', // Value
        ];

        let result = RadiusPacket::try_from(&buffer[..]).unwrap();

        assert_that!(result.code(), eq(RadiusPacketCode::AccessRequest));
        assert_that!(result.identifier(), eq(1));
        assert_that!(result.length(), eq(34));
        assert_that!(
            result.authenticator(),
            eq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        );
        assert_that!(
            result.attributes(),
            elements_are![
                eq(&RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::UserName,
                    b"hello".to_vec()
                )),
                eq(&RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::UserPassword,
                    b"world".to_vec()
                )),
            ]
        );
    }

    #[test]
    fn should_convert_from_byte_buffer_to_radius_packet_when_there_is_too_much_data() {
        let buffer = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            20, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
            1, 2, // Extra data
        ];

        let result = RadiusPacket::try_from(&buffer[..]).unwrap();

        assert_that!(result.code(), eq(RadiusPacketCode::AccessRequest));
        assert_that!(result.identifier(), eq(1));
        assert_that!(result.length(), eq(20));
        assert_that!(
            result.authenticator(),
            eq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        );
        assert_that!(result.attributes(), is_empty());
    }

    #[test]
    fn should_not_convert_from_byte_buffer_to_radius_packet_when_there_is_not_enough_data() {
        let buffer = vec![
            1,  // Access-Request
            1,  // Identifier
            0,  // Length MSB
            22, // Length LSB
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Authenticator
        ];

        let result = RadiusPacket::try_from(&buffer[..]).unwrap_err();

        assert_that!(result, matches_pattern!(RadiusPacketError::NotEnoughData));
    }
}
