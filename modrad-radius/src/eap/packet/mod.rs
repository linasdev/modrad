use std::fmt::Debug;
use crate::eap::packet::code::EapPacketCode;
use crate::eap::packet::data::EapPacketData;
use std::fmt::{Debug, Formatter};

pub mod code;
pub mod data;

const EAP_PACKET_HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub enum EapPacketError {
    NotEnoughData,
    TooMuchData,
}

pub struct EapPacket {
    identifier: u8,
    data: EapPacketData,
}

impl EapPacket {
    pub fn new(identifier: u8, data: EapPacketData) -> Self {
        Self { identifier, data }
    }

    pub fn identifier(&self) -> u8 {
        self.identifier
    }

    pub fn length(&self) -> usize {
        EAP_PACKET_HEADER_SIZE + self.data.length()
    }

    pub fn data(&self) -> &EapPacketData {
        &self.data
    }
}

impl TryFrom<&[u8]> for EapPacket {
    type Error = EapPacketError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < EAP_PACKET_HEADER_SIZE {
            return Err(EapPacketError::NotEnoughData);
        }

        let code = EapPacketCode::from(buffer[0]);
        let identifier = buffer[1];
        let length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;

        if buffer.len() < length {
            return Err(EapPacketError::NotEnoughData);
        }

        let data = EapPacketData::try_from((code, &buffer[EAP_PACKET_HEADER_SIZE..length]))?;

        Ok(Self { identifier, data })
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
