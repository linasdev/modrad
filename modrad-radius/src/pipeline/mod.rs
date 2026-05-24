use crate::eap::packet::EapPacketError;
use crate::pipeline::container::RadiusPacketContainer;

pub mod container;
pub mod metadata;

#[derive(Debug)]
pub enum RadiusPipelineError {
    EapPacket(EapPacketError),
}

pub trait RadiusPipeline {
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusPipelineError>;
}

impl From<EapPacketError> for RadiusPipelineError {
    fn from(error: EapPacketError) -> Self {
        RadiusPipelineError::EapPacket(error)
    }
}
