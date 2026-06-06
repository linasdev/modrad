use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::input::{RadiusInputError, RadiusInputPipelineStep};
use crate::radius::attribute::{RadiusPacketAttribute, RadiusPacketAttributeType};
use crate::radius::container::RadiusPacketInputContainer;
use crate::radius::metadata::RadiusPacketMetadata;
use hmac::{Hmac, KeyInit, Mac};
use log::info;
use md5::Md5;
use std::any::Any;
use async_trait::async_trait;

#[derive(Default)]
pub struct MessageAuthenticatorRadiusInputPipelineStep {
    secret: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageAuthenticatorStatus {
    NotFound,
    Valid,
    Invalid,
}

impl MessageAuthenticatorRadiusInputPipelineStep {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }
}

#[async_trait]
impl RadiusInputPipelineStep for MessageAuthenticatorRadiusInputPipelineStep {
    fn name(&self) -> String {
        "Message-Authenticator".to_string()
    }

    fn phase_code(&self) -> RadiusInputPhaseCode {
        RadiusInputPhaseCode::RadiusLayer
    }

    async fn process(
        &mut self,
        packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError> {
        let attributes = packet_container.packet().attributes();

        let message_authenticator = attributes
            .get(RadiusPacketAttributeType::MessageAuthenticator)
            .first()
            .map(|attribute| attribute.value());

        if let Some(message_authenticator) = message_authenticator {
            let mut packet_for_hashing = packet_container.packet().clone();
            let message_authenticator_attributes = packet_for_hashing
                .attributes_mut()
                .get_mut(RadiusPacketAttributeType::MessageAuthenticator);
            for message_authenticator_attribute in message_authenticator_attributes {
                *message_authenticator_attribute = RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::MessageAuthenticator,
                    vec![0; 16],
                );
            }
            let packet_bytes: Vec<u8> = packet_for_hashing.into();

            let mut mac = Hmac::<Md5>::new_from_slice(self.secret.as_slice()).unwrap();
            mac.update(&packet_bytes);
            let expected_message_authenticator = mac.finalize().into_bytes().0;

            if message_authenticator == expected_message_authenticator {
                info!(
                    "Valid Message-Authenticator attribute found in packet, adding MessageAuthenticatorStatus::Valid metadata"
                );
                packet_container.set_metadata(MessageAuthenticatorStatus::Valid);
            } else {
                info!(
                    "Invalid Message-Authenticator attribute found in packet, adding MessageAuthenticatorStatus::Invalid metadata"
                );
                packet_container.set_metadata(MessageAuthenticatorStatus::Invalid);
            }

            Ok(())
        } else {
            info!(
                "No Message-Authenticator attribute found in packet, adding MessageAuthenticatorStatus::NotFound metadata"
            );

            packet_container.set_metadata(MessageAuthenticatorStatus::NotFound);
            Ok(())
        }
    }
}

impl RadiusPacketMetadata for MessageAuthenticatorStatus {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
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
    async fn should_add_message_authenticator_status_not_found_metadata_to_container_when_there_is_no_message_authenticator_attribute()
     {
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

        let mut target = MessageAuthenticatorRadiusInputPipelineStep::new("secret");
        target.process(&mut container).await.unwrap();

        let result = container
            .get_metadata::<MessageAuthenticatorStatus>()
            .unwrap();

        assert_that!(result, eq(&MessageAuthenticatorStatus::NotFound),);
    }

    #[tokio::test]
    async fn should_add_message_authenticator_status_valid_metadata_to_container_when_there_is_a_valid_message_authenticator_attribute()
     {
        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::MessageAuthenticator,
            vec![
                184, 116, 44, 8, 53, 27, 50, 115, 163, 54, 94, 173, 149, 51, 12, 29,
            ],
        ));

        let mut container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusInputPipelineStep::new("secret");
        target.process(&mut container).await.unwrap();

        let result = container
            .get_metadata::<MessageAuthenticatorStatus>()
            .unwrap();

        assert_that!(result, eq(&MessageAuthenticatorStatus::Valid),);
    }

    #[tokio::test]
    async fn should_add_message_authenticator_status_valid_metadata_to_container_when_there_is_an_invalid_message_authenticator_attribute()
     {
        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::MessageAuthenticator,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        ));

        let mut container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusInputPipelineStep::new("secret");
        target.process(&mut container).await.unwrap();

        let result = container
            .get_metadata::<MessageAuthenticatorStatus>()
            .unwrap();

        assert_that!(result, eq(&MessageAuthenticatorStatus::Invalid),);
    }
}
