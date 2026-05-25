use crate::pipeline::RadiusPipelineError;
use crate::pipeline::container::RadiusPacketContainer;

pub mod message_authenticator;

pub trait RadiusPipelinePhase {
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusPipelineError>;
}
