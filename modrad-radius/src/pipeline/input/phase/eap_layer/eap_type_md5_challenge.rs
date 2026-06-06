use crate::chap::ChapPacketError;
use crate::chap::code::ChapPacketCode;
use crate::chap::data::ChapPacketData;
use crate::eap::EapPacket;
use crate::eap::data::{EapPacketData, EapPacketTypeData};
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::input::{RadiusInputError, RadiusInputPipelineStep};
use crate::radius::container::RadiusPacketInputContainer;
use crate::radius::metadata::RadiusPacketMetadata;
use async_trait::async_trait;
use log::{debug, info, trace, warn};
use pretty_hex::PrettyHex;
use std::any::Any;

#[derive(Default)]
pub struct EapTypeMD5ChallengeRadiusInputPipelineStep {}

#[derive(Debug, Clone)]
pub struct EapTypeDataMD5Challenge {
    pub value: Vec<u8>,
    pub name: Vec<u8>,
}

impl EapTypeMD5ChallengeRadiusInputPipelineStep {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl RadiusInputPipelineStep for EapTypeMD5ChallengeRadiusInputPipelineStep {
    fn name(&self) -> String {
        "MD5-Challenge".to_string()
    }

    fn phase_code(&self) -> RadiusInputPhaseCode {
        RadiusInputPhaseCode::EapLayer
    }

    async fn process(
        &mut self,
        input_packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError> {
        if let Some(eap_packet) = input_packet_container.get_metadata::<EapPacket>() {
            if let EapPacketData::Response {
                type_data: EapPacketTypeData::MD5Challenge(buffer),
            } = eap_packet.data()
            {
                debug!("EapPacket is of type MD5-Challenge, processing pipeline step");
                trace!(
                    "Parsing ChapPacketData::Response from buffer:\n{:?}",
                    buffer.hex_dump()
                );

                match ChapPacketData::try_from((ChapPacketCode::Response, &buffer[..])) {
                    Ok(ChapPacketData::Response { value, name }) => {
                        info!(
                            "MD5-Challenge type data contains a valid ChapPacketData::Response, adding EapTypeDataMD5Challenge metadata"
                        );
                        input_packet_container
                            .set_metadata(EapTypeDataMD5Challenge { value, name });

                        Ok(())
                    }
                    Ok(_) => unreachable!(),
                    Err(error) => match error {
                        ChapPacketError::NotEnoughData => {
                            warn!(
                                "ChapPacketData::Response's value length is shorter than it's value length field, skipping pipeline step processing"
                            );
                            Ok(())
                        }
                        _ => {
                            info!(
                                "MD5-Challenge type data contains no valid ChapPacketData::Response, skipping pipeline step processing"
                            );
                            Ok(())
                        }
                    },
                }
            } else {
                debug!("EapPacket is not of type MD5-Challenge, skipping pipeline step processing");
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

impl RadiusPacketMetadata for EapTypeDataMD5Challenge {
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
    use crate::eap::data::{EapPacketData, EapPacketTypeData};
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use crate::radius::code::RadiusPacketCode;
    use googletest::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_add_eap_type_data_md5_challenge_to_container_from_eap_packet_metadata() {
        let mut container = RadiusPacketInputContainer::new(
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
        container.set_metadata(EapPacket::new(
            220.into(),
            EapPacketData::Response {
                type_data: EapPacketTypeData::MD5Challenge(vec![
                    3, // value length
                    1, 2, 3, // value
                    4, 5, 6, // name
                ]),
            },
        ));

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).await.unwrap();

        let result = container.get_metadata::<EapTypeDataMD5Challenge>().unwrap();

        assert_that!(result.value, eq(&[1, 2, 3]));
        assert_that!(result.name, eq(&[4, 5, 6]));
    }

    #[tokio::test]
    async fn should_not_add_eap_type_data_md5_challenge_to_container_when_the_eap_packet_metadata_is_not_of_type_md5_challenge()
     {
        let mut container = RadiusPacketInputContainer::new(
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
        container.set_metadata(EapPacket::new(
            220.into(),
            EapPacketData::Response {
                type_data: EapPacketTypeData::GenericTokenCard(vec![1, 2, 3]),
            },
        ));

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).await.unwrap();

        assert_that!(
            container.has_metadata::<EapTypeDataMD5Challenge>(),
            is_false()
        );
    }

    #[tokio::test]
    async fn should_not_add_eap_type_data_md5_challenge_to_container_when_there_is_no_eap_packet_metadata()
     {
        let mut container = RadiusPacketInputContainer::new(
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

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).await.unwrap();

        assert_that!(
            container.has_metadata::<EapTypeDataMD5Challenge>(),
            is_false()
        );
    }
}
