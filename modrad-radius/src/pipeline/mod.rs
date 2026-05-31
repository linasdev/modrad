use crate::pipeline::mutability::RadiusPipelineMutability;
use log::info;
use std::any::TypeId;
use std::collections::BTreeMap;

pub mod handler;
pub mod input;
pub mod mutability;

pub trait RadiusPipelineStep<M, A>
where
    M: RadiusPipelineMutability,
    A: RadiusPipelineAcceptItem,
{
    type Error;
    type PhaseCode: RadiusPipelinePhaseCode;
    type TargetItem: RadiusPipelineTargetItem;

    fn name(&self) -> String;
    fn phase_code(&self) -> Self::PhaseCode;
    fn process(
        &mut self,
        target_item: M::Ref<'_, Self::TargetItem>,
    ) -> Result<RadiusPipelineStepAction<A>, Self::Error>;
}

pub trait RadiusPipelinePhaseCode: Ord + PartialOrd {
    fn name(&self) -> String;
}

pub trait RadiusPipelineAcceptItem {}

pub trait RadiusPipelineTargetItem {}

pub struct RadiusPipeline<M, P, T, A, E>
where
    M: RadiusPipelineMutability,
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTargetItem,
    A: RadiusPipelineAcceptItem,
{
    phases_by_code: BTreeMap<P, RadiusPipelinePhase<M, P, T, A, E>>,
}

pub struct RadiusPipelinePhase<M, P, T, A, E>
where
    M: RadiusPipelineMutability,
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTargetItem,
    A: RadiusPipelineAcceptItem,
{
    steps_by_type: BTreeMap<
        TypeId,
        Box<dyn RadiusPipelineStep<M, A, Error = E, PhaseCode = P, TargetItem = T>>,
    >,
}

#[derive(Debug)]
pub enum RadiusPipelineStepAction<A> {
    DiscardTarget,
    NextStep,
    AcceptStep(A),
}

impl<M, P, T, A, E> RadiusPipeline<M, P, T, A, E>
where
    M: RadiusPipelineMutability,
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTargetItem,
    A: RadiusPipelineAcceptItem,
{
    pub fn new() -> Self {
        Self {
            phases_by_code: BTreeMap::new(),
        }
    }

    pub fn insert_step<
        S: RadiusPipelineStep<M, A, Error = E, PhaseCode = P, TargetItem = T> + 'static,
    >(
        &mut self,
        pipeline_step: S,
    ) -> Result<&mut Self, S> {
        let pipeline_phase = self
            .phases_by_code
            .entry(pipeline_step.phase_code())
            .or_insert_with(|| RadiusPipelinePhase {
                steps_by_type: BTreeMap::new(),
            });

        let step_type_id = TypeId::of::<S>();

        if pipeline_phase.steps_by_type.contains_key(&step_type_id) {
            Err(pipeline_step)
        } else {
            pipeline_phase
                .steps_by_type
                .insert(step_type_id, Box::new(pipeline_step));
            Ok(self)
        }
    }

    pub fn process(&mut self, mut target_item: M::Ref<'_, T>) -> Result<Option<A>, E> {
        for (phase_code, stage) in self.phases_by_code.iter_mut() {
            info!("Processing pipeline phase {}", phase_code.name());

            for (_, pipeline_step) in stage.steps_by_type.iter_mut() {
                info!("Processing pipeline step {}", pipeline_step.name());

                let current_target_item = M::reborrow(&mut target_item);
                match pipeline_step.process(current_target_item)? {
                    RadiusPipelineStepAction::AcceptStep(accept_item) => {
                        info!("Final pipeline action is AcceptStep(A), returning Some(A)");
                        return Ok(Some(accept_item));
                    }
                    RadiusPipelineStepAction::DiscardTarget => {
                        info!("Final pipeline action is DiscardTarget, returning None");
                        return Ok(None);
                    }
                    RadiusPipelineStepAction::NextStep => continue,
                }
            }
        }

        Ok(None)
    }
}
