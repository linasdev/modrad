use crate::packet::container::RadiusPacketContainer;
use crate::pipeline::RadiusPipelineStepAction;
use crate::pipeline::handler::phase::RadiusHandlerPhaseCode;
use crate::pipeline::handler::{
    RadiusHandlerError, RadiusHandlerPipelineStep, RadiusPacketWithDestination,
};
use crate::pipeline::input::message_authenticator::MessageAuthenticatorStatus;
use log::info;

pub struct MessageAuthenticatorRadiusHandlerPipelineStep {}

impl MessageAuthenticatorRadiusHandlerPipelineStep {
    pub fn new() -> Self {
        Self {}
    }
}

impl RadiusHandlerPipelineStep for MessageAuthenticatorRadiusHandlerPipelineStep {
    fn name(&self) -> String {
        "Message-Authenticator".to_string()
    }

    fn phase_code(&self) -> RadiusHandlerPhaseCode {
        RadiusHandlerPhaseCode::Validation
    }

    fn process(
        &mut self,
        packet_container: &RadiusPacketContainer,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketWithDestination>, RadiusHandlerError> {
        match packet_container.get_metadata::<MessageAuthenticatorStatus>() {
            Some(MessageAuthenticatorStatus::Valid) => {
                info!(
                    "RADIUS packet has a valid Message-Authenticator attribute, processing next handler(s)"
                );
                Ok(RadiusPipelineStepAction::NextStep)
            }
            Some(MessageAuthenticatorStatus::Invalid) => {
                info!(
                    "RADIUS packet has an invalid Message-Authenticator attribute, discarding packet"
                );
                Ok(RadiusPipelineStepAction::DiscardTarget)
            }
            _ => {
                info!("RADIUS packet is missing a Message-Authenticator attribute");
                Ok(RadiusPipelineStepAction::NextStep)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::RadiusPacket;
    use crate::packet::attribute::RadiusPacketAttributes;
    use crate::packet::code::RadiusPacketCode;
    use crate::peer::RadiusPeer;
    use googletest::prelude::*;

    #[test]
    fn should_return_next_step_when_message_authenticator_valid_metadata_exists() {
        let mut container = RadiusPacketContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            },
        );

        container.set_metadata(MessageAuthenticatorStatus::Valid);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }

    #[test]
    fn should_return_discard_packet_when_message_authenticator_invalid_metadata_exists() {
        let mut container = RadiusPacketContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            },
        );

        container.set_metadata(MessageAuthenticatorStatus::Invalid);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).unwrap();

        assert_that!(
            result,
            matches_pattern!(RadiusPipelineStepAction::DiscardTarget)
        );
    }

    #[test]
    fn should_return_next_step_when_message_authenticator_not_found_metadata_exists() {
        let mut container = RadiusPacketContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            },
        );

        container.set_metadata(MessageAuthenticatorStatus::NotFound);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }

    #[test]
    fn should_return_next_step_when_message_authenticator_metadata_does_not_exist() {
        let container = RadiusPacketContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            },
        );

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }
}
