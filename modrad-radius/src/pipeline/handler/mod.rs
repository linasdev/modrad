use crate::packet::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use crate::pipeline::handler::phase::RadiusHandlerPhaseCode;
use crate::pipeline::{RadiusPipelineStep, RadiusPipelineStepAction, RadiusPipelineTarget};

pub mod message_authenticator;
pub mod phase;

#[derive(Debug)]
pub enum RadiusHandlerError {}

pub struct RadiusHandlerPipelineTarget;

pub trait RadiusHandlerPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusHandlerPhaseCode;
    fn process(
        &mut self,
        packet_container: &RadiusPacketInputContainer,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketOutputContainer>, RadiusHandlerError>;
}

impl<S> RadiusPipelineStep<RadiusHandlerPipelineTarget, RadiusPacketOutputContainer> for S
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
        target: <RadiusHandlerPipelineTarget as RadiusPipelineTarget>::Ref<'_>,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketOutputContainer>, Self::Error> {
        self.process(target)
    }
}

impl RadiusPipelineTarget for RadiusHandlerPipelineTarget {
    type Ref<'t> = &'t RadiusPacketInputContainer;

    fn reborrow<'t1, 't2>(t: &'t1 mut Self::Ref<'t2>) -> Self::Ref<'t1>
    where
        't2: 't1,
    {
        *t
    }
}
