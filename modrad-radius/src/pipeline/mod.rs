use crate::eap::packet::EapPacketError;

pub mod container;
pub mod metadata;
pub mod phase;

#[derive(Debug)]
pub enum RadiusPipelineError {
    EapPacket(EapPacketError),
}

impl From<EapPacketError> for RadiusPipelineError {
    fn from(error: EapPacketError) -> Self {
        RadiusPipelineError::EapPacket(error)
    }
}
