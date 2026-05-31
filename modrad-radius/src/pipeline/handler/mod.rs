use crate::packet::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use crate::pipeline::handler::phase::RadiusHandlerPhaseCode;
use crate::pipeline::mutability::{RadiusPipelineMutability, SharedRadiusPipeline};
use crate::pipeline::{RadiusPipelineAcceptItem, RadiusPipelineStep, RadiusPipelineStepAction};

pub mod message_authenticator;
pub mod phase;

#[derive(Debug)]
pub enum RadiusHandlerError {}

pub trait RadiusHandlerPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusHandlerPhaseCode;
    fn process(
        &mut self,
        packet_container: &RadiusPacketInputContainer,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketOutputContainer>, RadiusHandlerError>;
}

impl<S> RadiusPipelineStep<SharedRadiusPipeline, RadiusPacketInputContainer, RadiusPacketOutputContainer> for S
where
    S: RadiusHandlerPipelineStep,
{
    type Error = RadiusHandlerError;
    type PhaseCode = RadiusHandlerPhaseCode;

    fn name(&self) -> String {
        self.name()
    }

    fn phase_code(&self) -> Self::PhaseCode {
        self.phase_code()
    }

    fn process(
        &mut self,
        target_item: <SharedRadiusPipeline as RadiusPipelineMutability>::Ref<'_, RadiusPacketInputContainer>,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketOutputContainer>, Self::Error> {
        self.process(target_item)
    }
}
