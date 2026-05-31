use crate::pipeline::RadiusPipelinePhaseCode;
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord)]
pub enum RadiusOutputPhaseCode {
    MessageAuthenticator,
    ResponseAuthenticator,
    Other(usize),
}

impl RadiusOutputPhaseCode {
    fn position(&self) -> isize {
        match self {
            RadiusOutputPhaseCode::MessageAuthenticator => isize::MIN + 0,
            RadiusOutputPhaseCode::ResponseAuthenticator => isize::MIN + 1,
            RadiusOutputPhaseCode::Other(code) => *code as isize,
        }
    }
}

impl RadiusPipelinePhaseCode for RadiusOutputPhaseCode {
    fn name(&self) -> String {
        match self {
            RadiusOutputPhaseCode::MessageAuthenticator => "MESSAGE_AUTHENTICATOR".to_string(),
            RadiusOutputPhaseCode::ResponseAuthenticator => "RESPONSE_AUTHENTICATOR".to_string(),
            RadiusOutputPhaseCode::Other(code) => format!("OTHER ({code})"),
        }
    }
}

impl PartialOrd for RadiusOutputPhaseCode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position().partial_cmp(&other.position())
    }
}
