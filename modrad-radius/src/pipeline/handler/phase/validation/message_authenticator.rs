use crate::pipeline::RadiusPipelineStepAction;
use crate::pipeline::handler::phase::RadiusHandlerPhaseCode;
use crate::pipeline::handler::{RadiusHandlerError, RadiusHandlerPipelineStep};
use crate::pipeline::input::phase::radius_layer::message_authenticator::MessageAuthenticatorStatus;
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use async_trait::async_trait;
use log::{info, warn};

pub struct MessageAuthenticatorRadiusHandlerPipelineStep {}

impl MessageAuthenticatorRadiusHandlerPipelineStep {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RadiusHandlerPipelineStep for MessageAuthenticatorRadiusHandlerPipelineStep {
    fn name(&self) -> String {
        "Message-Authenticator".to_string()
    }

    fn phase_code(&self) -> RadiusHandlerPhaseCode {
        RadiusHandlerPhaseCode::Validation
    }

    async fn process(
        &mut self,
        packet_container: &RadiusPacketInputContainer,
    ) -> Result<RadiusPipelineStepAction<RadiusPacketOutputContainer>, RadiusHandlerError> {
        match packet_container.get_metadata::<MessageAuthenticatorStatus>() {
            Some(MessageAuthenticatorStatus::Valid) => {
                info!(
                    "RADIUS packet has a valid Message-Authenticator attribute, processing next handler(s)"
                );
                Ok(RadiusPipelineStepAction::NextStep)
            }
            Some(MessageAuthenticatorStatus::Invalid) => {
                warn!(
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
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use crate::radius::code::RadiusPacketCode;
    use googletest::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_return_next_step_when_message_authenticator_valid_metadata_exists() {
        let mut container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        container.set_metadata(MessageAuthenticatorStatus::Valid);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).await.unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }

    #[tokio::test]
    async fn should_return_discard_packet_when_message_authenticator_invalid_metadata_exists() {
        let mut container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        container.set_metadata(MessageAuthenticatorStatus::Invalid);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).await.unwrap();

        assert_that!(
            result,
            matches_pattern!(RadiusPipelineStepAction::DiscardTarget)
        );
    }

    #[tokio::test]
    async fn should_return_next_step_when_message_authenticator_not_found_metadata_exists() {
        let mut container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        container.set_metadata(MessageAuthenticatorStatus::NotFound);

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).await.unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }

    #[tokio::test]
    async fn should_return_next_step_when_message_authenticator_metadata_does_not_exist() {
        let container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusHandlerPipelineStep::new();
        let result = target.process(&container).await.unwrap();

        assert_that!(result, matches_pattern!(RadiusPipelineStepAction::NextStep));
    }
}
