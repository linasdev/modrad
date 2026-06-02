use crate::chap::packet::code::ChapPacketCode;
use crate::chap::packet::data::ChapPacketData;
use crate::packet::metadata::RadiusPacketMetadata;
use std::any::Any;
use std::fmt::{Debug, Formatter};

pub mod code;
pub mod data;

const CHAP_PACKET_HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub enum ChapPacketError {
    NotEnoughData,
    InvalidCode,
}

pub struct ChapPacket {
    identifier: u8,
    data: ChapPacketData,
}

impl ChapPacket {
    pub fn new(identifier: u8, data: ChapPacketData) -> Self {
        Self { identifier, data }
    }

    pub fn code(&self) -> ChapPacketCode {
        self.data.code()
    }

    pub fn identifier(&self) -> u8 {
        self.identifier
    }

    pub fn length(&self) -> usize {
        CHAP_PACKET_HEADER_SIZE + self.data.length()
    }

    pub fn data(&self) -> &ChapPacketData {
        &self.data
    }
}

impl From<ChapPacket> for Vec<u8> {
    fn from(packet: ChapPacket) -> Self {
        let mut buffer = Vec::with_capacity(packet.length());

        buffer.push(packet.code().into()); // byte 0
        buffer.push(packet.identifier); // byte 1

        for byte in u16::to_be_bytes(packet.length() as u16) {
            buffer.push(byte); // bytes 2 & 3
        }

        buffer.extend_from_slice(&Vec::from(packet.data)); // bytes 4 - length

        buffer
    }
}

impl TryFrom<&[u8]> for ChapPacket {
    type Error = ChapPacketError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < CHAP_PACKET_HEADER_SIZE {
            return Err(ChapPacketError::NotEnoughData);
        }

        let code = ChapPacketCode::try_from(buffer[0]).map_err(|_| ChapPacketError::InvalidCode)?;
        let identifier = buffer[1];
        let length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;

        if buffer.len() < length {
            return Err(ChapPacketError::NotEnoughData);
        }

        let data = ChapPacketData::try_from((code, &buffer[CHAP_PACKET_HEADER_SIZE..length]))?;

        Ok(Self { identifier, data })
    }
}

impl RadiusPacketMetadata for ChapPacket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Debug for ChapPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChapPacket")
            .field("identifier", &self.identifier())
            .field("length", &self.length())
            .field("data", self.data())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn should_convert_from_chap_packet_challenge_to_byte_buffer() {
        let packet = ChapPacket::new(
            0,
            ChapPacketData::Challenge {
                value: vec![1, 2, 3],
                name: vec![4, 5, 6],
            },
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            1, // code
            0, // identifier
            0, 11, // length
            3,  // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_response_to_byte_buffer() {
        let packet = ChapPacket::new(
            0,
            ChapPacketData::Response {
                value: vec![1, 2, 3],
                name: vec![4, 5, 6],
            },
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            2, // code
            0, // identifier
            0, 11, // length
            3,  // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_success_to_byte_buffer() {
        let packet = ChapPacket::new(
            0,
            ChapPacketData::Success {
                message: vec![1, 2, 3],
            },
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            3, // code
            0, // identifier
            0, 7, // length
            1, 2, 3, // message
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_failure_to_byte_buffer() {
        let packet = ChapPacket::new(
            0,
            ChapPacketData::Failure {
                message: vec![1, 2, 3],
            },
        );

        let result = Vec::from(packet);

        let expected_result = vec![
            4, // code
            0, // identifier
            0, 7, // length
            1, 2, 3, // message
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_byte_buffer_to_chap_packet_challenge() {
        let buffer = vec![
            1, // code
            0, // identifier
            0, 11, // length
            3,  // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        let result = ChapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacket {
                identifier: eq(&0),
                data: matches_pattern!(ChapPacketData::Challenge {
                    value: eq(&[1, 2, 3]),
                    name: eq(&[4, 5, 6]),
                })
            })
        )
    }

    #[test]
    fn should_convert_byte_buffer_to_chap_packet_response() {
        let buffer = vec![
            2, // code
            0, // identifier
            0, 11, // length
            3,  // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        let result = ChapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacket {
                identifier: eq(&0),
                data: matches_pattern!(ChapPacketData::Response {
                    value: eq(&[1, 2, 3]),
                    name: eq(&[4, 5, 6]),
                })
            })
        )
    }

    #[test]
    fn should_convert_byte_buffer_to_chap_packet_success() {
        let buffer = vec![
            3, // code
            0, // identifier
            0, 7, // length
            1, 2, 3, // message
        ];

        let result = ChapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacket {
                identifier: eq(&0),
                data: matches_pattern!(ChapPacketData::Success {
                    message: eq(&[1, 2, 3]),
                })
            })
        )
    }

    #[test]
    fn should_convert_byte_buffer_to_chap_packet_failure() {
        let buffer = vec![
            4, // code
            0, // identifier
            0, 7, // length
            1, 2, 3, // message
        ];

        let result = ChapPacket::try_from(&buffer[..]).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacket {
                identifier: eq(&0),
                data: matches_pattern!(ChapPacketData::Failure {
                    message: eq(&[1, 2, 3]),
                })
            })
        )
    }

    #[test]
    fn should_not_convert_from_byte_buffer_to_chap_packet_challenge_when_there_is_not_enough_data()
    {
        let buffer = vec![
            1, // code
            0, // identifier
            0, 11, // length
            3,  // value length
            1, 2, 3, // value
            4, 5, // missing byte
        ];

        let result = ChapPacket::try_from(&buffer[..]).unwrap_err();

        assert_that!(result, matches_pattern!(ChapPacketError::NotEnoughData))
    }
}
