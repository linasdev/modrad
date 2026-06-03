use crate::chap::packet::data::ChapPacketData;
use crate::eap::packet::EapPacket;
use crate::eap::packet::data::{EapPacketData, EapPacketTypeData};
use crate::identifier_pool::EapIdentifierPool;
use crate::packet::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::output::{RadiusOutputError, RadiusOutputPipelineStep};
use log::info;
use std::time::Instant;

pub struct ChapPacketRadiusOutputPipelineStep {
    eap_identifier_pool: EapIdentifierPool,
}

impl ChapPacketRadiusOutputPipelineStep {
    pub fn new(eap_identifier_pool: EapIdentifierPool) -> Self {
        Self {
            eap_identifier_pool,
        }
    }
}

impl RadiusOutputPipelineStep for ChapPacketRadiusOutputPipelineStep {
    fn name(&self) -> String {
        "ChapPacket".to_string()
    }

    fn phase_code(&self) -> RadiusOutputPhaseCode {
        RadiusOutputPhaseCode::EapLayer
    }

    fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        _input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError> {
        if let Some(chap_packet_data) = output_packet_container.take_metadata::<ChapPacketData>() {
            info!(
                "ChapPacketData metadata found in output packet container, checking for EapPacket metadata"
            );

            if !output_packet_container.has_metadata::<EapPacket>() {
                info!(
                    "No EapPacket metadata found in output packet container, setting EapPacket metadata"
                );

                let eap_identifier = self
                    .eap_identifier_pool
                    .allocate(Instant::now())
                    .ok_or(RadiusOutputError::NoIdentifierAvailable)?;

                let eap_packet = EapPacket::new(
                    eap_identifier,
                    EapPacketData::Request {
                        type_data: EapPacketTypeData::MD5Challenge(Vec::from(chap_packet_data)),
                    },
                );

                output_packet_container.set_metadata(eap_packet);
            } else {
                info!(
                    "EapPacket metadata found in output packet container, leaving EapPacket metadata unchanged"
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
    use crate::chap::packet::data::ChapPacketData;
    use crate::eap::packet::code::EapPacketCode;
    use crate::eap::packet::data::EapPacketData;
    use crate::packet::RadiusPacket;
    use crate::packet::attribute::RadiusPacketAttributes;
    use crate::packet::code::RadiusPacketCode;
    use crate::peer::RadiusPeer;
    use googletest::prelude::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn should_add_eap_packet_metadata_to_container_from_chap_packet_data_metadata() {
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
        output_container.set_metadata(ChapPacketData::Challenge {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        });

        let mut target =
            ChapPacketRadiusOutputPipelineStep::new(EapIdentifierPool::new(Duration::from_mins(1)));
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container.get_metadata::<EapPacket>().unwrap();

        assert_that!(result.code(), eq(EapPacketCode::Request));
        assert_that!(result.identifier(), eq(0));
        assert_that!(
            result.data(),
            matches_pattern!(EapPacketData::Request {
                type_data: matches_pattern!(EapPacketTypeData::MD5Challenge(&[
                    3,  // value length
                    1, 2, 3, // value
                    4, 5, 6, // name
                ])),
            })
        );
    }

    #[test]
    fn should_not_add_eap_packet_metadata_to_container_when_there_already_is_eap_packet_metadata() {
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
        output_container.set_metadata(ChapPacketData::Challenge {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        });
        output_container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::Identity(vec![106, 111, 104, 110, 95, 100, 111, 101]),
            },
        ));

        let mut target =
            ChapPacketRadiusOutputPipelineStep::new(EapIdentifierPool::new(Duration::from_mins(1)));
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container.get_metadata::<EapPacket>().unwrap();

        assert_that!(result.code(), eq(EapPacketCode::Response));
        assert_that!(result.identifier(), eq(220));
        assert_that!(
            result.data(),
            matches_pattern!(EapPacketData::Response {
                type_data: matches_pattern!(EapPacketTypeData::Identity(&[
                    106, 111, 104, 110, 95, 100, 111, 101
                ])),
            })
        );
    }

    #[test]
    fn should_not_add_eap_packet_metadata_to_container_when_there_is_no_chap_packet_data_metadata() {
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

        let mut target =
            ChapPacketRadiusOutputPipelineStep::new(EapIdentifierPool::new(Duration::from_mins(1)));
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container.get_metadata::<EapPacket>();
        assert_that!(result, none());
    }
}
