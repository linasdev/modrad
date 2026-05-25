use crate::eap::packet::EapPacketError;
use std::string::FromUtf8Error;

pub mod container;
pub mod metadata;
pub mod phase;

#[derive(Debug)]
pub enum RadiusPipelineError {
    EapPacket(EapPacketError),
    FromUtf8(FromUtf8Error),
}

impl From<EapPacketError> for RadiusPipelineError {
    fn from(error: EapPacketError) -> Self {
        RadiusPipelineError::EapPacket(error)
    }
}

impl From<FromUtf8Error> for RadiusPipelineError {
    fn from(error: FromUtf8Error) -> Self {
        RadiusPipelineError::FromUtf8(error)
    }
}
