use crate::packet::RadiusPacket;
use crate::packet::container::RadiusPacketContainer;
use crate::peer::RadiusPeer;
use crate::pipeline::handler::phase::RadiusHandlerPhaseCode;
use crate::pipeline::mutability::{RadiusPipelineMutability, SharedRadiusPipeline};
use crate::pipeline::{RadiusPipelineAcceptItem, RadiusPipelineStep, RadiusPipelineStepAction};

pub mod message_authenticator;
pub mod phase;

#[derive(Debug)]
pub enum RadiusHandlerError {}

#[derive(Debug)]
pub struct RadiusPacketWithDestination {
    packet: RadiusPacket,
    peer: RadiusPeer,
}

pub trait RadiusHandlerPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusHandlerPhaseCode;
    fn process(
        &mut self,
        packet_container: &RadiusPacketContainer,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketWithDestination>, RadiusHandlerError>;
}

impl<S> RadiusPipelineStep<SharedRadiusPipeline, RadiusPacketWithDestination> for S
where
    S: RadiusHandlerPipelineStep,
{
    type Error = RadiusHandlerError;
    type PhaseCode = RadiusHandlerPhaseCode;
    type TargetItem = RadiusPacketContainer;

    fn name(&self) -> String {
        self.name()
    }

    fn phase_code(&self) -> Self::PhaseCode {
        self.phase_code()
    }

    fn process(
        &mut self,
        target_item: <SharedRadiusPipeline as RadiusPipelineMutability>::Ref<'_, Self::TargetItem>,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketWithDestination>, Self::Error> {
        self.process(target_item)
    }
}

impl RadiusPipelineAcceptItem for RadiusPacketWithDestination {}
