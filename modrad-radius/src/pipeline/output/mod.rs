use crate::packet::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use crate::pipeline::mutability::{MutatingRadiusPipeline, RadiusPipelineMutability};
use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::{RadiusPipelineStep, RadiusPipelineStepAction, RadiusPipelineTargetItem};

pub mod message_authenticator;
pub mod phase;
pub mod response_authenticator;

#[derive(Debug)]
pub enum RadiusOutputError {}

pub trait RadiusOutputPipelineStep {
    fn name(&self) -> String;
    fn phase_code(&self) -> RadiusOutputPhaseCode;
    fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError>;
}

impl<S>
    RadiusPipelineStep<
        MutatingRadiusPipeline,
        (
            &mut RadiusPacketOutputContainer,
            &RadiusPacketInputContainer,
        ),
        (),
    > for S
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
        target_item: <MutatingRadiusPipeline as RadiusPipelineMutability>::Ref<
            '_,
            (
                &mut RadiusPacketOutputContainer,
                &RadiusPacketInputContainer,
            ),
        >,
    ) -> Result<RadiusPipelineStepAction<()>, Self::Error> {
        let (output_packet_container, input_packet_container) = target_item;
        self.process(output_packet_container, input_packet_container)?;
        Ok(RadiusPipelineStepAction::NextStep)
    }
}

impl RadiusPipelineTargetItem
    for (
        &mut RadiusPacketOutputContainer,
        &RadiusPacketInputContainer,
    )
{
}
