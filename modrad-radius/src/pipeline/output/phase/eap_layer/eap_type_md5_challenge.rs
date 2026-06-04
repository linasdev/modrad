use crate::chap::data::ChapPacketData;
use crate::eap::EapPacket;
use crate::eap::data::{EapPacketData, EapPacketTypeData};
use crate::identifier_pool::EapIdentifierPool;
use crate::pipeline::input::phase::eap_layer::eap_type_md5_challenge::EapTypeDataMD5Challenge;
use crate::pipeline::output::phase::RadiusOutputPhaseCode;
use crate::pipeline::output::{RadiusOutputError, RadiusOutputPipelineStep};
use crate::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use log::info;
use std::time::Instant;

pub struct EapTypeMD5ChallengeRadiusOutputPipelineStep {
    eap_identifier_pool: EapIdentifierPool,
}

impl EapTypeMD5ChallengeRadiusOutputPipelineStep {
    pub fn new(eap_identifier_pool: EapIdentifierPool) -> Self {
        Self {
            eap_identifier_pool,
        }
    }
}

impl RadiusOutputPipelineStep for EapTypeMD5ChallengeRadiusOutputPipelineStep {
    fn name(&self) -> String {
        "MD5-Challenge".to_string()
    }

    fn phase_code(&self) -> RadiusOutputPhaseCode {
        RadiusOutputPhaseCode::EapLayer
    }

    fn process(
        &mut self,
        output_packet_container: &mut RadiusPacketOutputContainer,
        _input_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusOutputError> {
        if let Some(eap_type_data) =
            output_packet_container.take_metadata::<EapTypeDataMD5Challenge>()
        {
            info!(
                "EapTypeDataMD5Challenge metadata found in output packet container, processing pipeline step"
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
                        type_data: EapPacketTypeData::MD5Challenge(Vec::from(
                            ChapPacketData::Challenge {
                                value: eap_type_data.value,
                                name: eap_type_data.name,
                            },
                        )),
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
    use crate::eap::code::EapPacketCode;
    use crate::eap::data::EapPacketData;
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use crate::radius::code::RadiusPacketCode;
    use googletest::prelude::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn should_add_eap_packet_metadata_to_container_from_eap_type_data_md5_challenge() {
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
        output_container.set_metadata(EapTypeDataMD5Challenge {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        });

        let mut target = EapTypeMD5ChallengeRadiusOutputPipelineStep::new(EapIdentifierPool::new(
            Duration::from_mins(1),
        ));
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
                    3, // value length
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
        output_container.set_metadata(EapTypeDataMD5Challenge {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        });
        output_container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::Identity(vec![106, 111, 104, 110, 95, 100, 111, 101]),
            },
        ));

        let mut target = EapTypeMD5ChallengeRadiusOutputPipelineStep::new(EapIdentifierPool::new(
            Duration::from_mins(1),
        ));
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
    fn should_not_add_eap_packet_metadata_to_container_when_there_is_no_eap_type_data_md5_challenge()
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

        let mut target = EapTypeMD5ChallengeRadiusOutputPipelineStep::new(EapIdentifierPool::new(
            Duration::from_mins(1),
        ));
        target
            .process(&mut output_container, &input_container)
            .unwrap();

        let result = output_container.get_metadata::<EapPacket>();
        assert_that!(result, none());
    }
}
