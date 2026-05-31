use log::info;
use std::any::TypeId;
use std::collections::BTreeMap;

pub mod handler;
pub mod input;
pub mod output;

pub trait RadiusPipelineStep<T, A>
where
    T: RadiusPipelineTarget,
    A: RadiusPipelineAcceptItem,
{
    type Error;
    type PhaseCode: RadiusPipelinePhaseCode;

    fn name(&self) -> String;
    fn phase_code(&self) -> Self::PhaseCode;
    fn process(&mut self, target: T::Ref<'_>) -> Result<RadiusPipelineStepAction<A>, Self::Error>;
}

pub trait RadiusPipelinePhaseCode: Ord + PartialOrd {
    fn name(&self) -> String;
}

pub trait RadiusPipelineAcceptItem {}

pub trait RadiusPipelineTarget {
    type Ref<'t>;

    fn reborrow<'t1, 't2>(t: &'t1 mut Self::Ref<'t2>) -> Self::Ref<'t1>
    where
        't2: 't1;
}

pub struct RadiusPipeline<P, T, A, E>
where
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTarget,
    A: RadiusPipelineAcceptItem,
{
    name: String,
    phases_by_code: BTreeMap<P, RadiusPipelinePhase<P, T, A, E>>,
}

pub struct RadiusPipelinePhase<P, T, A, E>
where
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTarget,
    A: RadiusPipelineAcceptItem,
{
    steps_by_type: BTreeMap<TypeId, Box<dyn RadiusPipelineStep<T, A, Error = E, PhaseCode = P>>>,
}

#[derive(Debug)]
pub enum RadiusPipelineStepAction<A> {
    DiscardTarget,
    NextStep,
    AcceptStep(A),
}

impl<P, T, A, E> RadiusPipeline<P, T, A, E>
where
    P: RadiusPipelinePhaseCode,
    T: RadiusPipelineTarget,
    A: RadiusPipelineAcceptItem,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            phases_by_code: BTreeMap::new(),
        }
    }

    pub fn insert_step<S: RadiusPipelineStep<T, A, Error = E, PhaseCode = P> + 'static>(
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

    pub fn process(&mut self, mut target: T::Ref<'_>) -> Result<Option<A>, E> {
        info!("Processing pipeline {}", self.name.as_str());

        for (phase_code, stage) in self.phases_by_code.iter_mut() {
            info!(
                "Processing pipeline phase {}/{}",
                self.name.as_str(),
                phase_code.name()
            );

            for (_, pipeline_step) in stage.steps_by_type.iter_mut() {
                info!(
                    "Processing pipeline step {}/{}/{}",
                    self.name.as_str(),
                    phase_code.name(),
                    pipeline_step.name()
                );

                let current_target = T::reborrow(&mut target);
                match pipeline_step.process(current_target)? {
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

        info!("No pipeline step returned a final action, returning None");

        Ok(None)
    }
}

impl RadiusPipelineAcceptItem for () {}
