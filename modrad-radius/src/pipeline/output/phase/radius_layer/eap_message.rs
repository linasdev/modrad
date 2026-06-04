use crate::eap::EapPacket;
use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::output::{RadiusOutputError, RadiusOutputPipelineStep};
use crate::radius::attribute::{RadiusPacketAttribute, RadiusPacketAttributeType};
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use log::info;

#[derive(Default)]
pub struct EapMessageRadiusOutputPipelineStep {}

impl EapMessageRadiusOutputPipelineStep {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RadiusOutputPipelineStep for EapMessageRadiusOutputPipelineStep {
    fn name(&self) -> String {
        "EAP-Message".to_string()
    }

    fn phase_code(&self) -> RadiusOutputPhaseCode {
        RadiusOutputPhaseCode::RadiusLayer
    }

    fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        _input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError> {
        if let Some(eap_packet) = output_packet_container.take_metadata::<EapPacket>() {
            info!("EapPacket metadata found in output packet container, processing pipeline step");

            let packet = output_packet_container.packet_mut();
            let attributes = packet.attributes_mut();

            if !attributes.has(RadiusPacketAttributeType::EAPMessage) {
                info!(
                    "No EAP-Message attribute found in output packet, setting EAP-Message attribute"
                );

                attributes.push(RadiusPacketAttribute::from_tag_and_value(
                    RadiusPacketAttributeType::EAPMessage,
                    Vec::from(eap_packet),
                ));
            } else {
                info!(
                    "EAP-Message attribute found in output packet, leaving EAP-Message attribute unchanged"
                );
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eap::data::{EapPacketData, EapPacketTypeData};
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use crate::radius::code::RadiusPacketCode;
    use googletest::prelude::*;
    use std::sync::Arc;

    #[test]
    fn should_add_eap_message_attribute_to_packet_from_eap_message_metadata() {
        let input_container = RadiusPacketInputContainer::new(
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
        let mut output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessChallenge,
                1,
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );
        output_container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::Identity(vec![106, 111, 104, 110, 95, 100, 111, 101]),
            },
        ));

        let mut target = EapMessageRadiusOutputPipelineStep::new();
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container
            .packet()
            .attributes()
            .get(RadiusPacketAttributeType::EAPMessage);

        assert_that!(
            result,
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::EAPMessage,
                vec![2, 220, 0, 13, 1, 106, 111, 104, 110, 95, 100, 111, 101],
            )),]
        );
    }

    #[test]
    fn should_not_add_eap_message_attribute_to_packet_when_there_already_is_an_eap_message_attribute()
     {
        let input_container = RadiusPacketInputContainer::new(
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

        let mut attributes = RadiusPacketAttributes::new();
        attributes.push(RadiusPacketAttribute::from_tag_and_value(
            RadiusPacketAttributeType::EAPMessage,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        ));
        let mut output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessChallenge,
                1,
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                attributes,
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );
        output_container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::Identity(vec![106, 111, 104, 110, 95, 100, 111, 101]),
            },
        ));

        let mut target = EapMessageRadiusOutputPipelineStep::new();
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container
            .packet()
            .attributes()
            .get(RadiusPacketAttributeType::EAPMessage);

        assert_that!(
            result,
            elements_are![eq(&&RadiusPacketAttribute::from_tag_and_value(
                RadiusPacketAttributeType::EAPMessage,
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            )),]
        );
    }

    #[test]
    fn should_not_add_eap_message_attribute_to_packet_when_there_is_no_eap_message_metadata() {
        let input_container = RadiusPacketInputContainer::new(
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
        let mut output_container = RadiusPacketOutputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessChallenge,
                1,
                [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let mut target = EapMessageRadiusOutputPipelineStep::new();
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container
            .packet()
            .attributes()
            .get(RadiusPacketAttributeType::EAPMessage);

        assert_that!(result, is_empty());
    }
}
