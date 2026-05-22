use crate::tag_length_value::{TagLengthValue, TagLengthValueError};

const RADIUS_PACKET_HEADER_SIZE: usize = 20;

#[derive(Debug)]
pub enum RadiusPacketError {
    NotEnoughData,
    TooMuchData,
    TagLengthValue(TagLengthValueError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RadiusPacketCode {
    AccessRequest,
    AccessAccept,
    AccessReject,
    AccountingRequest,
    AccountingResponse,
    AccessChallenge,
    Other(u8),
}

pub struct RadiusPacket {
    code: RadiusPacketCode,
    identifier: u8,
    length: usize,
    authenticator: [u8; 16],
    attributes: Vec<RadiusPacketAttribute>,
}

impl From<TagLengthValueError> for RadiusPacketError {
    fn from(error: TagLengthValueError) -> Self {
        RadiusPacketError::TagLengthValue(error)
    }
}

impl From<u8> for RadiusPacketCode {
    fn from(value: u8) -> Self {
        match value {
            1 => RadiusPacketCode::AccessRequest,
            2 => RadiusPacketCode::AccessAccept,
            3 => RadiusPacketCode::AccessReject,
            4 => RadiusPacketCode::AccountingRequest,
            5 => RadiusPacketCode::AccountingResponse,
            11 => RadiusPacketCode::AccessChallenge,
            _ => RadiusPacketCode::Other(value),
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RadiusPacketAttributeType {
    UserName,
    UserPassword,
    Other(u8),
}

pub type RadiusPacketAttribute = TagLengthValue<RadiusPacketAttributeType>;

impl From<u8> for RadiusPacketAttributeType {
    fn from(value: u8) -> Self {
        match value {
            1 => RadiusPacketAttributeType::UserName,
            2 => RadiusPacketAttributeType::UserPassword,
            _ => RadiusPacketAttributeType::Other(value),
        }
    }
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

    pub fn authenticator(&self) -> &[u8] {
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

        let length = u16::from_be_bytes([
            packet_data[2],
            packet_data[3],
        ]) as usize;

        if packet_data.len() < length {
            return Err(RadiusPacketError::NotEnoughData);
        }

        if packet_data.len() > length {
            return Err(RadiusPacketError::TooMuchData);
        }

        let authenticator: [u8; 16] = packet_data[4..20].try_into().unwrap();

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
