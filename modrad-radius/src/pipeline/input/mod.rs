use crate::eap::packet::EapPacketError;
use crate::packet::container::RadiusPacketInputContainer;
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::mutability::{MutatingRadiusPipeline, RadiusPipelineMutability};
use crate::pipeline::{RadiusPipelineStep, RadiusPipelineStepAction};
use std::string::FromUtf8Error;

pub mod message_authenticator;
pub mod phase;

#[derive(Debug)]
pub enum RadiusInputError {
    EapPacket(EapPacketError),
    FromUtf8(FromUtf8Error),
}

pub trait RadiusInputPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusInputPhaseCode;
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError>;
}

impl<S> RadiusPipelineStep<MutatingRadiusPipeline, RadiusPacketInputContainer, ()> for S
where
    S: RadiusInputPipelineStep,
{
    type Error = RadiusInputError;
    type PhaseCode = RadiusInputPhaseCode;

    fn name(&self) -> String {
        self.name()
    }

    fn phase_code(&self) -> Self::PhaseCode {
        self.phase_code()
    }

    fn process(
        &mut self,
        target_item: <MutatingRadiusPipeline as RadiusPipelineMutability>::Ref<
            '_,
            RadiusPacketInputContainer,
        >,
    ) -> Result<RadiusPipelineStepAction<()>, Self::Error> {
        self.process(target_item)?;
        Ok(RadiusPipelineStepAction::NextStep)
    }
}

impl From<EapPacketError> for RadiusInputError {
    fn from(error: EapPacketError) -> Self {
        RadiusInputError::EapPacket(error)
    }
}

impl From<FromUtf8Error> for RadiusInputError {
    fn from(error: FromUtf8Error) -> Self {
        RadiusInputError::FromUtf8(error)
    }
}
