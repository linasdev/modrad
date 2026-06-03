use crate::pipeline::RadiusPipelinePhaseCode;
use std::cmp::Ordering;

pub mod eap_layer;
pub mod message_authenticator;
pub mod radius_layer;
pub mod response_authenticator;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord)]
pub enum RadiusOutputPhaseCode {
    EapLayer,
    RadiusLayer,
    MessageAuthenticator,
    ResponseAuthenticator,
    Other(usize),
}

impl RadiusOutputPhaseCode {
    fn position(&self) -> isize {
        match self {
            RadiusOutputPhaseCode::EapLayer => isize::MIN + 0,
            RadiusOutputPhaseCode::RadiusLayer => isize::MIN + 1,
            RadiusOutputPhaseCode::MessageAuthenticator => isize::MIN + 2,
            RadiusOutputPhaseCode::ResponseAuthenticator => isize::MIN + 3,
            RadiusOutputPhaseCode::Other(code) => *code as isize,
        }
    }
}

impl RadiusPipelinePhaseCode for RadiusOutputPhaseCode {
    fn name(&self) -> String {
        match self {
            RadiusOutputPhaseCode::EapLayer => "EAP_LAYER".to_string(),
            RadiusOutputPhaseCode::RadiusLayer => "RADIUS_LAYER".to_string(),
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
