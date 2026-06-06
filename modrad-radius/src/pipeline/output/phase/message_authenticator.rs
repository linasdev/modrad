use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::output::{RadiusOutputError, RadiusOutputPipelineStep};
use crate::radius::attribute::{RadiusPacketAttribute, RadiusPacketAttributeType};
use crate::radius::code::RadiusPacketCode;
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use async_trait::async_trait;
use hmac::{Hmac, KeyInit, Mac};
use md5::Md5;

#[derive(Default)]
pub struct MessageAuthenticatorRadiusOutputPipelineStep {
    secret: Vec<u8>,
}

impl MessageAuthenticatorRadiusOutputPipelineStep {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }
}

#[async_trait]
impl RadiusOutputPipelineStep for MessageAuthenticatorRadiusOutputPipelineStep {
    fn name(&self) -> String {
        "Sign".to_string()
    }

    fn phase_code(&self) -> RadiusOutputPhaseCode {
        RadiusOutputPhaseCode::MessageAuthenticator
    }

    async fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError> {
        let packet = output_packet_container.packet_mut();

        {
            let attributes = packet.attributes_mut();
            attributes.remove_all(RadiusPacketAttributeType::MessageAuthenticator);
            attributes.push(RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                vec![0; 16],
            ));
        }

        let mut packet_clone = packet.clone();

        // RFC 3579 states:
        // For Access-Challenge, Access-Accept, and Access-Reject packets,
        // the Message-Authenticator is calculated as follows, using the
        // Request-Authenticator from the Access-Request this packet is in reply to:
        // Message-Authenticator = HMAC-MD5 (Type, Identifier, Length, Request Authenticator, Attributes)
        match packet.code() {
            RadiusPacketCode::AccessChallenge
            | RadiusPacketCode::AccessAccept
            | RadiusPacketCode::AccessReject => {
                packet_clone.set_authenticator(input_packet_container.packet().authenticator());
            }
            _ => {}
        }

        let packet_bytes: Vec<u8> = packet_clone.into();

        let mut mac = Hmac::<Md5>::new_from_slice(self.secret.as_slice()).unwrap();
        mac.update(&packet_bytes);
        let message_authenticator = mac.finalize().into_bytes().0.to_vec();

        let attributes = packet.attributes_mut();
        let message_authenticator_attributes =
            attributes.get_mut(RadiusPacketAttributeType::MessageAuthenticator);
        for message_authenticator_attribute in message_authenticator_attributes {
            *message_authenticator_attribute = RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                message_authenticator,
            );
            break;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use googletest::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_add_message_authenticator_attribute_when_code_is_access_request() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0.into(),
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserName,
            b"user_name".to_vec(),
        ));
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserPassword,
            b"user_password".to_vec(),
        ));

        let mut packet_output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0.into(),
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusOutputPipelineStep::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        );
        assert_that!(
            packet
                .attributes()
                .get(RadiusPacketAttributeType::MessageAuthenticator),
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                vec![
                    202, 139, 184, 235, 215, 39, 175, 248, 214, 241, 218, 110, 130, 72, 22, 134
                ],
            ))],
        );
    }

    #[tokio::test]
    async fn should_add_message_authenticator_attribute_when_code_is_access_challenge() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0.into(),
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserName,
            b"user_name".to_vec(),
        ));
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserPassword,
            b"user_password".to_vec(),
        ));

        let mut packet_output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessChallenge,
                0.into(),
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusOutputPipelineStep::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        );
        assert_that!(
            packet
                .attributes()
                .get(RadiusPacketAttributeType::MessageAuthenticator),
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                vec![
                    188, 64, 97, 171, 155, 146, 235, 41, 208, 173, 79, 155, 240, 11, 20, 38
                ],
            ))],
        );
    }

    #[tokio::test]
    async fn should_add_message_authenticator_attribute_when_code_is_access_accept() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0.into(),
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserName,
            b"user_name".to_vec(),
        ));
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserPassword,
            b"user_password".to_vec(),
        ));

        let mut packet_output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessAccept,
                0.into(),
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusOutputPipelineStep::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        );
        assert_that!(
            packet
                .attributes()
                .get(RadiusPacketAttributeType::MessageAuthenticator),
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                vec![
                    3, 42, 18, 219, 123, 65, 87, 87, 41, 219, 31, 145, 41, 141, 226, 36
                ],
            ))],
        );
    }

    #[tokio::test]
    async fn should_add_message_authenticator_attribute_when_code_is_access_reject() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0.into(),
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserName,
            b"user_name".to_vec(),
        ));
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::UserPassword,
            b"user_password".to_vec(),
        ));

        let mut packet_output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessReject,
                0.into(),
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = MessageAuthenticatorRadiusOutputPipelineStep::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        );
        assert_that!(
            packet
                .attributes()
                .get(RadiusPacketAttributeType::MessageAuthenticator),
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::MessageAuthenticator,
                vec![
                    169, 153, 165, 203, 211, 157, 211, 150, 136, 255, 134, 193, 41, 193, 237, 152
                ],
            ))],
        );
    }
}
