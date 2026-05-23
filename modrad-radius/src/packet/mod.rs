use crate::packet::attribute::RadiusPacketAttribute;
use crate::packet::code::RadiusPacketCode;
use crate::tag_length_value::TagLengthValueError;

pub mod attribute;
pub mod code;

const RADIUS_PACKET_HEADER_SIZE: usize = 20;

#[derive(Debug)]
pub enum RadiusPacketError {
    NotEnoughData,
    TooMuchData,
    TagLengthValue(TagLengthValueError),
}

pub struct RadiusPacket {
    code: RadiusPacketCode,
    identifier: u8,
    authenticator: [u8; 16],
    attributes: Vec<RadiusPacketAttribute>,
}

impl RadiusPacket {
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

impl TryFrom<Vec<u8>> for RadiusPacket {
    type Error = RadiusPacketError;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        if buffer.len() < RADIUS_PACKET_HEADER_SIZE {
            return Err(RadiusPacketError::NotEnoughData);
        }

        let code = RadiusPacketCode::from(buffer[0]);
        let identifier = buffer[1];

        let length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;

        if buffer.len() < length {
            return Err(RadiusPacketError::NotEnoughData);
        }

        if buffer.len() > length {
            return Err(RadiusPacketError::TooMuchData);
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

impl From<TagLengthValueError> for RadiusPacketError {
    fn from(error: TagLengthValueError) -> Self {
        RadiusPacketError::TagLengthValue(error)
    }
}
