use std::any::Any;
use hmac::Hmac;
use crate::eap::packet::{EapPacket, EapPacketError};
use crate::packet::attribute::{RadiusPacketAttribute, RadiusPacketAttributeType};
use crate::pipeline::RadiusPipelineError;
use crate::pipeline::container::RadiusPacketContainer;
use crate::pipeline::phase::RadiusPipelinePhase;
use log::info;
use md5::digest::{KeyInit, Mac};
use md5::Md5;
use crate::pipeline::metadata::RadiusPacketMetadata;

#[derive(Default)]
pub struct MessageAuthenticatorRadiusPipelinePhase {
    secret: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageAuthenticatorStatus {
    NotFound,
    Valid,
    Invalid,
}

impl MessageAuthenticatorRadiusPipelinePhase {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }
}

impl RadiusPipelinePhase for MessageAuthenticatorRadiusPipelinePhase {
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusPipelineError> {
        info!("Processing pipeline phase");

        let attributes = packet_container.packet().attributes();

        let message_authenticator = attributes.get(RadiusPacketAttributeType::MessageAuthenticator)
            .first()
            .map(|attribute| attribute.value());

        if let Some(message_authenticator) = message_authenticator {
            let mut packet_for_hashing = packet_container.packet().clone();
            let message_authenticator_attributes = packet_for_hashing.attributes_mut().get_mut(RadiusPacketAttributeType::MessageAuthenticator);
            for message_authenticator_attribute in message_authenticator_attributes {
                *message_authenticator_attribute = RadiusPacketAttribute::from_tag_and_value(RadiusPacketAttributeType::MessageAuthenticator, vec![0; 16]);
            }
            let packet_bytes: Vec<u8> = packet_for_hashing.into();

            let mut mac = Hmac::<Md5>::new_from_slice(self.secret.as_slice()).unwrap();
            mac.update(&packet_bytes);
            let expected_message_authenticator = mac.finalize().into_bytes().0;

            if message_authenticator == expected_message_authenticator {
                info!("Valid Message-Authenticator attribute found in packet, adding MessageAuthenticatorStatus::Valid metadata");
                packet_container.set_metadata(MessageAuthenticatorStatus::Valid);
            } else {
                info!("Invalid Message-Authenticator attribute found in packet, adding MessageAuthenticatorStatus::Invalid metadata");
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
