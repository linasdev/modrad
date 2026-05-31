use crate::eap::packet::{EapPacket, EapPacketError};
use crate::packet::attribute::RadiusPacketAttributeType;
use crate::packet::container::RadiusPacketInputContainer;
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::input::{RadiusInputError, RadiusInputPipelineStep};
use log::{debug, info};

#[derive(Default)]
pub struct EapPacketRadiusInputPipelineStep {}

impl EapPacketRadiusInputPipelineStep {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RadiusInputPipelineStep for EapPacketRadiusInputPipelineStep {
    fn name(&self) -> String {
        "EAP-Message".to_string()
    }

    fn phase_code(&self) -> RadiusInputPhaseCode {
        RadiusInputPhaseCode::RadiusAttribute
    }

    fn process(
        &mut self,
        packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError> {
        let attributes = packet_container.packet().attributes();

        if !attributes.has(RadiusPacketAttributeType::EAPMessage) {
            debug!("No EAP-Message attribute found in packet, skipping pipeline step processing");
            return Ok(());
        }

        info!("EAP-Message attribute(s) found in packet, processing pipeline step");

        let eap_message: Vec<u8> = attributes
            .get(RadiusPacketAttributeType::EAPMessage)
            .into_iter()
            .flat_map(|attribute| attribute.value())
            .copied()
            .collect();

        let eap_packet = match EapPacket::try_from(&eap_message[..]) {
            Ok(eap_packet) => eap_packet,
            Err(error) => {
                match error {
                    EapPacketError::NotEnoughData => {
                        info!(
                            "EAP-Message attribute(s) total length is shorter than it's length field, skipping pipeline step processing"
                        );
                        return Ok(());
                    }
                    EapPacketError::InvalidCode => {
                        info!(
                            "EAP-Message attribute(s) contains an invalid EAP code, skipping pipeline step processing"
                        );
                        return Ok(());
                    }
                }

                #[allow(unreachable_code)]
                return Err(RadiusInputError::EapPacket(error));
            }
        };

        info!("Valid EAP-Message attribute(s) found in packet, adding EapPacket metadata");
        packet_container.set_metadata(eap_packet);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eap::packet::code::EapPacketCode;
    use crate::eap::packet::data::{EapPacketData, EapPacketType, EapPacketTypeData};
    use crate::packet::RadiusPacket;
    use crate::packet::attribute::{RadiusPacketAttribute, RadiusPacketAttributes};
    use crate::packet::code::RadiusPacketCode;
    use crate::peer::RadiusPeer;
    use googletest::prelude::*;
    use std::sync::Arc;

    #[test]
    fn should_add_eap_packet_metadata_to_container_from_eap_message_attribute() {
        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::EAPMessage,
            vec![2, 220, 0, 13, 1, 106, 111, 104, 110, 95, 100, 111, 101],
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

        let mut target = EapPacketRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        let result = container.get_metadata::<EapPacket>().unwrap();

        assert_that!(result.code(), eq(EapPacketCode::Response));
        assert_that!(result.identifier(), eq(220));
        assert_that!(result.length(), eq(13));

        let EapPacketData::Response { type_data } = result.data() else {
            unreachable!()
        };

        assert_that!(type_data.packet_type(), eq(EapPacketType::Identity));
        assert_that!(type_data.length(), eq(8));

        let EapPacketTypeData::Identity(buffer) = type_data else {
            unreachable!()
        };

        assert_that!(buffer, eq(&[106, 111, 104, 110, 95, 100, 111, 101]))
    }

    #[test]
    fn should_not_add_eap_packet_metadata_to_container_when_there_is_no_eap_message_attribute() {
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

        let mut target = EapPacketRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        assert_that!(container.has_metadata::<EapPacket>(), is_false());
    }
}
