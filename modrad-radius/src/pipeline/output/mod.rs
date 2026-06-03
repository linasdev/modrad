use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::{RadiusPipelineStep, RadiusPipelineStepAction, RadiusPipelineTarget};
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};

pub mod phase;

#[derive(Debug)]
pub enum RadiusOutputError {
    NoIdentifierAvailable,
}

pub struct RadiusOutputPipelineTarget;

pub trait RadiusOutputPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusOutputPhaseCode;
    fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError>;
}

impl<S> RadiusPipelineStep<RadiusOutputPipelineTarget, ()> for S
where
    S: RadiusOutputPipelineStep,
{
    type Error = RadiusOutputError;
    type PhaseCode = RadiusOutputPhaseCode;

    fn name(&self) -> String {
        self.name()
    }

    fn phase_code(&self) -> Self::PhaseCode {
        self.phase_code()
    }

    fn process(
        &mut self,
        target: <RadiusOutputPipelineTarget as RadiusPipelineTarget>::Ref<'_>,
    ) -> Result<RadiusPipelineStepAction<()>, Self::Error> {
        self.process(target.0, target.1)?;
        Ok(RadiusPipelineStepAction::NextStep)
    }
}

impl RadiusPipelineTarget for RadiusOutputPipelineTarget {
    type Ref<'t> = (
        &'t mut RadiusPacketOutputContainer,
        &'t RadiusPacketInputContainer,
    );

    fn reborrow<'t1, 't2>(t: &'t1 mut Self::Ref<'t2>) -> Self::Ref<'t1>
    where
        't2: 't1,
    {
        (t.0, t.1)
    }
}
