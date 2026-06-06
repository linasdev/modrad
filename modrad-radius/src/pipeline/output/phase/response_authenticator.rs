use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::output::{RadiusOutputError, RadiusOutputPipelineStep};
use crate::radius::code::RadiusPacketCode;
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use async_trait::async_trait;
use md5::{Digest, Md5};

#[derive(Default)]
pub struct ResponseAuthenticatorRadiusPacketTransformer {
    secret: Vec<u8>,
}

impl ResponseAuthenticatorRadiusPacketTransformer {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }
}

#[async_trait]
impl RadiusOutputPipelineStep for ResponseAuthenticatorRadiusPacketTransformer {
    fn name(&self) -> String {
        "Sign".to_string()
    }

    fn phase_code(&self) -> RadiusOutputPhaseCode {
        RadiusOutputPhaseCode::ResponseAuthenticator
    }

    async fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError> {
        let packet = output_packet_container.packet_mut();

        // RFC 2865 states:
        // The value of the Authenticator field in Access-Accept, Access-
        // Reject, and Access-Challenge packets is called the Response
        // Authenticator, and contains a one-way MD5 hash calculated over
        // a stream of octets consisting of: the RADIUS radius, beginning
        // with the Code field, including the Identifier, the Length, the
        // Request Authenticator field from the Access-Request radius, and
        // the response Attributes, followed by the shared secret.  That
        // is, ResponseAuth = MD5(Code+ID+Length+RequestAuth+Attributes+Secret)
        // where + denotes concatenation.
        match packet.code() {
            RadiusPacketCode::AccessAccept
            | RadiusPacketCode::AccessReject
            | RadiusPacketCode::AccessChallenge => {
                let mut packet_clone = packet.clone();
                packet_clone.set_authenticator(input_packet_container.packet().authenticator());

                let mut packet_bytes: Vec<u8> = packet_clone.into();
                packet_bytes.extend_from_slice(self.secret.as_slice());

                let response_authenticator = Md5::digest(packet_bytes.as_slice()).0;
                packet.set_authenticator(&response_authenticator);
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::{
        RadiusPacketAttribute, RadiusPacketAttributeType, RadiusPacketAttributes,
    };
    use googletest::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_not_add_response_authenticator_when_code_is_access_request() {
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

        let mut target = ResponseAuthenticatorRadiusPacketTransformer::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        );
        assert_that!(packet.attributes().iter().count(), eq(2));
    }

    #[tokio::test]
    async fn should_add_response_authenticator_when_code_is_access_accept() {
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

        let mut target = ResponseAuthenticatorRadiusPacketTransformer::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[
                54, 119, 89, 99, 24, 52, 189, 59, 224, 10, 103, 150, 239, 46, 11, 127
            ]),
        );
        assert_that!(packet.attributes().iter().count(), eq(2));
    }

    #[tokio::test]
    async fn should_add_response_authenticator_when_code_is_access_reject() {
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

        let mut target = ResponseAuthenticatorRadiusPacketTransformer::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[
                42, 87, 175, 80, 152, 230, 42, 162, 222, 0, 202, 107, 168, 201, 221, 32,
            ]),
        );
        assert_that!(packet.attributes().iter().count(), eq(2));
    }

    #[tokio::test]
    async fn should_add_response_authenticator_when_code_is_access_challenge() {
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

        let mut target = ResponseAuthenticatorRadiusPacketTransformer::new("secret");
        target
            .process(&mut packet_output_container, &packet_input_container)
            .await
            .unwrap();

        let packet = packet_output_container.packet();
        assert_that!(
            packet.authenticator(),
            eq(&[
                241, 195, 87, 186, 222, 148, 195, 227, 8, 130, 32, 78, 225, 145, 167, 143,
            ]),
        );
        assert_that!(packet.attributes().iter().count(), eq(2));
    }
}
