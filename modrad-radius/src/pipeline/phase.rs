use crate::pipeline::container::RadiusPacketContainer;
use crate::pipeline::RadiusPipelineError;

pub trait RadiusPipelinePhase {
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusPipelineError>;
}
