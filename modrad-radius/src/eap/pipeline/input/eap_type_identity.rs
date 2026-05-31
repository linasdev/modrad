use crate::eap::packet::EapPacket;
use crate::eap::packet::data::EapPacketTypeData;
use crate::packet::container::RadiusPacketContainer;
use crate::packet::metadata::RadiusPacketMetadata;
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::input::{RadiusInputError, RadiusInputPipelineStep};
use log::{debug, info};
use std::any::Any;

#[derive(Default)]
pub struct EapTypeIdentityRadiusInputPipelineStep {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EapTypeDataIdentity {
    UserName(String),
}

impl EapTypeIdentityRadiusInputPipelineStep {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RadiusInputPipelineStep for EapTypeIdentityRadiusInputPipelineStep {
    fn name(&self) -> String {
        "EAP-Message/Identity".to_string()
    }

    fn phase_code(&self) -> RadiusInputPhaseCode {
        RadiusInputPhaseCode::EapType
    }

    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusInputError> {
        if let Some(eap_packet) = packet_container.get_metadata::<EapPacket>() {
            if let Some(EapPacketTypeData::Identity(buffer)) = eap_packet.data().type_data() {
                info!("EapPacket is of type Identity, processing pipeline step");

                if let Ok(user_name) = String::from_utf8(buffer.clone()) {
                    info!(
                        "Identity type data contains a valid UTF-8 encoded user name, adding EapTypeDataIdentity::UserName metadata"
                    );
                    packet_container.set_metadata(EapTypeDataIdentity::UserName(user_name));
                    Ok(())
                } else {
                    info!(
                        "Identity type data contains no valid identity, skipping pipeline step processing"
                    );
                    Ok(())
                }
            } else {
                debug!("EapPacket is not of type Identity, skipping pipeline step processing");
                Ok(())
            }
        } else {
            debug!(
                "No EapPacket metadata found in packet container, skipping pipeline step processing"
            );
            Ok(())
        }
    }
}

impl RadiusPacketMetadata for EapTypeDataIdentity {
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
    use crate::eap::packet::data::EapPacketData;
    use crate::packet::RadiusPacket;
    use crate::packet::attribute::RadiusPacketAttributes;
    use crate::packet::code::RadiusPacketCode;
    use crate::peer::RadiusPeer;
    use googletest::prelude::*;

    #[test]
    fn should_add_eap_type_data_identity_metadata_to_container_from_eap_packet_metadata() {
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
        container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Request {
                type_data: EapPacketTypeData::Identity(vec![106, 111, 104, 110, 95, 100, 111, 101]),
            },
        ));

        let mut target = EapTypeIdentityRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        let result = container.get_metadata::<EapTypeDataIdentity>().unwrap();

        assert_that!(
            result,
            eq(&EapTypeDataIdentity::UserName("john_doe".to_string()))
        );
    }

    #[test]
    fn should_not_add_eap_type_data_identity_metadata_to_container_when_there_is_no_eap_packet_metadata()
     {
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

        let mut target = EapTypeIdentityRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        assert_that!(container.has_metadata::<EapTypeDataIdentity>(), is_false());
    }
}
