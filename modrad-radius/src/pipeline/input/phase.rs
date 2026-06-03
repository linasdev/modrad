use crate::pipeline::RadiusPipelinePhaseCode;
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord)]
pub enum RadiusInputPhaseCode {
    RadiusLayer,
    EapLayer,
    Other(usize),
}

impl RadiusInputPhaseCode {
    fn position(&self) -> isize {
        match self {
            RadiusInputPhaseCode::RadiusLayer => isize::MIN + 0,
            RadiusInputPhaseCode::EapLayer => isize::MIN + 1,
            RadiusInputPhaseCode::Other(code) => *code as isize,
        }
    }
}

impl RadiusPipelinePhaseCode for RadiusInputPhaseCode {
    fn name(&self) -> String {
        match self {
            RadiusInputPhaseCode::RadiusLayer => "RADIUS_LAYER".to_string(),
            RadiusInputPhaseCode::EapLayer => "EAP_LAYER".to_string(),
            RadiusInputPhaseCode::Other(code) => format!("OTHER ({code})"),
        }
    }
}

impl PartialOrd for RadiusInputPhaseCode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position().partial_cmp(&other.position())
    }
}
