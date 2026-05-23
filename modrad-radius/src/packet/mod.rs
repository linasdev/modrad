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
    length: usize,
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
        self.length
    }

    pub fn authenticator(&self) -> &[u8; 16] {
        &self.authenticator
    }

    pub fn attributes(&self) -> &[RadiusPacketAttribute] {
        &self.attributes
    }
}

impl TryFrom<Vec<u8>> for RadiusPacket {
    type Error = RadiusPacketError;

    fn try_from(packet_data: Vec<u8>) -> Result<Self, Self::Error> {
        if packet_data.len() < RADIUS_PACKET_HEADER_SIZE {
            return Err(RadiusPacketError::NotEnoughData);
        }

        let code = RadiusPacketCode::from(packet_data[0]);
        let identifier = packet_data[1];

        let length = u16::from_be_bytes([packet_data[2], packet_data[3]]) as usize;

        if packet_data.len() < length {
            return Err(RadiusPacketError::NotEnoughData);
        }

        if packet_data.len() > length {
            return Err(RadiusPacketError::TooMuchData);
        }

        let authenticator: [u8; 16] = packet_data[4..RADIUS_PACKET_HEADER_SIZE]
            .try_into()
            .unwrap();

        let mut offset = RADIUS_PACKET_HEADER_SIZE;
        let mut attributes = Vec::new();
        while offset < length {
            let attribute = RadiusPacketAttribute::try_from(&packet_data[offset..])?;
            offset += attribute.length();
            attributes.push(attribute);
        }

        Ok(Self {
            code,
            identifier,
            length,
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
