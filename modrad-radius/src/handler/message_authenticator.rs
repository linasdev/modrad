use log::info;
use crate::handler::action::RadiusHandlerAction;
use crate::handler::{RadiusHandler, RadiusHandlerError};
use crate::pipeline::container::RadiusPacketContainer;
use crate::pipeline_phase::message_authenticator::MessageAuthenticatorStatus;

pub struct MessageAuthenticatorRadiusHandler {}

impl MessageAuthenticatorRadiusHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl RadiusHandler for MessageAuthenticatorRadiusHandler {
    fn handle<'p>(
        &mut self,
        packet_container: &'p RadiusPacketContainer,
    ) -> Result<RadiusHandlerAction<'p>, RadiusHandlerError> {
        match packet_container.get_metadata::<MessageAuthenticatorStatus>() {
            Some(MessageAuthenticatorStatus::Valid) => {
                info!("RADIUS packet has a valid Message-Authenticator attribute, processing next handler(s)");
                Ok(RadiusHandlerAction::NextHandler)
            }
            Some(MessageAuthenticatorStatus::Invalid) => {
                info!("RADIUS packet has an invalid Message-Authenticator attribute, discarding packet");
                Ok(RadiusHandlerAction::DiscardPacket)
            }
            _ => {
                info!("RADIUS packet is missing a Message-Authenticator attribute");
                Ok(RadiusHandlerAction::NextHandler)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use crate::handler::action::RadiusHandlerAction;
    use crate::handler::message_authenticator::MessageAuthenticatorRadiusHandler;
    use crate::handler::RadiusHandler;
    use crate::packet::attribute::RadiusPacketAttributes;
    use crate::packet::code::RadiusPacketCode;
    use crate::packet::RadiusPacket;
    use crate::peer::RadiusPeer;
    use crate::pipeline::container::RadiusPacketContainer;
    use crate::pipeline_phase::message_authenticator::MessageAuthenticatorStatus;

    #[test]
    fn should_return_next_handler_when_message_authenticator_valid_metadata_exists() {
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

        let mut target = MessageAuthenticatorRadiusHandler::new();
        let result = target.handle(&mut container).unwrap();

        assert_that!(result, matches_pattern!(RadiusHandlerAction::NextHandler));
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

        let mut target = MessageAuthenticatorRadiusHandler::new();
        let result = target.handle(&mut container).unwrap();

        assert_that!(result, matches_pattern!(RadiusHandlerAction::DiscardPacket));
    }

    #[test]
    fn should_return_next_handler_when_message_authenticator_not_found_metadata_exists() {
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

        let mut target = MessageAuthenticatorRadiusHandler::new();
        let result = target.handle(&mut container).unwrap();

        assert_that!(result, matches_pattern!(RadiusHandlerAction::NextHandler));
    }

    #[test]
    fn should_return_next_handler_when_message_authenticator_metadata_does_not_exist() {
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

        let mut target = MessageAuthenticatorRadiusHandler::new();
        let result = target.handle(&mut container).unwrap();

        assert_that!(result, matches_pattern!(RadiusHandlerAction::NextHandler));
    }
}
