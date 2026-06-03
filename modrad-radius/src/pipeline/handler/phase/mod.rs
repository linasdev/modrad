use crate::pipeline::RadiusPipelinePhaseCode;
use std::cmp::Ordering;

pub mod eap;
pub mod validation;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord)]
pub enum RadiusHandlerPhaseCode {
    Validation,
    Eap,
    Other(usize),
}

impl RadiusHandlerPhaseCode {
    fn position(&self) -> isize {
        match self {
            RadiusHandlerPhaseCode::Validation => isize::MIN + 0,
            RadiusHandlerPhaseCode::Eap => isize::MIN + 1,
            RadiusHandlerPhaseCode::Other(code) => *code as isize,
        }
    }
}

impl RadiusPipelinePhaseCode for RadiusHandlerPhaseCode {
    fn name(&self) -> String {
        match self {
            RadiusHandlerPhaseCode::Validation => "VALIDATION".to_string(),
            RadiusHandlerPhaseCode::Eap => "EAP".to_string(),
            RadiusHandlerPhaseCode::Other(code) => format!("OTHER ({code})"),
        }
    }
}

impl PartialOrd for RadiusHandlerPhaseCode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position().partial_cmp(&other.position())
    }
}
