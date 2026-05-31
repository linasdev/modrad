use crate::eap::packet::EapPacketError;
use crate::packet::container::RadiusPacketInputContainer;
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::{RadiusPipelineStep, RadiusPipelineStepAction, RadiusPipelineTarget};
use std::string::FromUtf8Error;

pub mod message_authenticator;
pub mod phase;

#[derive(Debug)]
pub enum RadiusInputError {
    EapPacket(EapPacketError),
    FromUtf8(FromUtf8Error),
}

pub struct RadiusInputPipelineTarget;

pub trait RadiusInputPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusInputPhaseCode;
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError>;
}

impl<S> RadiusPipelineStep<RadiusInputPipelineTarget, ()> for S
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
        target: <RadiusInputPipelineTarget as RadiusPipelineTarget>::Ref<'_>,
    ) -> Result<RadiusPipelineStepAction<()>, Self::Error> {
        self.process(target)?;
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

impl RadiusPipelineTarget for RadiusInputPipelineTarget {
    type Ref<'t> = &'t mut RadiusPacketInputContainer;

    fn reborrow<'t1, 't2>(t: &'t1 mut Self::Ref<'t2>) -> Self::Ref<'t1>
    where
        't2: 't1,
    {
        *t
    }
}
