use crate::chap::ChapPacketError;
use crate::chap::code::ChapPacketCode;
use crate::chap::data::ChapPacketData;
use crate::eap::EapPacket;
use crate::eap::data::{EapPacketData, EapPacketTypeData};
use crate::pipeline::input::phase::RadiusInputPhaseCode;
use crate::pipeline::input::{RadiusInputError, RadiusInputPipelineStep};
use crate::radius::container::RadiusPacketInputContainer;
use log::{debug, info, trace, warn};
use pretty_hex::PrettyHex;

#[derive(Default)]
pub struct EapTypeMD5ChallengeRadiusInputPipelineStep {}

impl EapTypeMD5ChallengeRadiusInputPipelineStep {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RadiusInputPipelineStep for EapTypeMD5ChallengeRadiusInputPipelineStep {
    fn name(&self) -> String {
        "MD5-Challenge".to_string()
    }

    fn phase_code(&self) -> RadiusInputPhaseCode {
        RadiusInputPhaseCode::EapLayer
    }

    fn process(
        &mut self,
        input_packet_container: &mut RadiusPacketInputContainer,
    ) -> Result<(), RadiusInputError> {
        if let Some(eap_packet) = input_packet_container.get_metadata::<EapPacket>() {
            debug!("EapPacket metadata found in input packet container, checking type data");

            if let EapPacketData::Response {
                type_data: EapPacketTypeData::MD5Challenge(buffer),
            } = eap_packet.data()
            {
                debug!("EapPacket is of type MD5-Challenge, processing pipeline step");
                trace!(
                    "Parsing ChapPacketData::Response from buffer:\n{:?}",
                    buffer.hex_dump()
                );

                let chap_packet_data = match ChapPacketData::try_from((
                    ChapPacketCode::Response,
                    &buffer[..],
                )) {
                    Ok(chap_packet_data) => chap_packet_data,
                    Err(error) => {
                        match error {
                            ChapPacketError::NotEnoughData => {
                                warn!(
                                    "ChapPacketData::Response's value length is shorter than it's value length field, skipping pipeline step processing"
                                );
                                return Ok(());
                            }
                            _ => {}
                        }

                        #[allow(unreachable_code)]
                        return Err(RadiusInputError::ChapPacket(error));
                    }
                };

                info!(
                    "Valid ChapPacketData::Response found in packet, adding ChapPacketData::Response metadata"
                );
                input_packet_container.set_metadata(chap_packet_data);
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
    fn should_add_chap_packet_data_metadata_to_container_from_eap_packet_metadata() {
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
        container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::MD5Challenge(vec![
                    3, // value length
                    1, 2, 3, // value
                    4, 5, 6, // name
                ]),
            },
        ));

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        let ChapPacketData::Response { value, name } =
            container.get_metadata::<ChapPacketData>().unwrap()
        else {
            unreachable!()
        };

        assert_that!(value, eq(&[1, 2, 3]));
        assert_that!(name, eq(&[4, 5, 6]));
    }

    #[test]
    fn should_not_add_chap_packet_data_metadata_to_container_when_the_eap_packet_metadata_is_not_of_type_md5_challenge()
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
        container.set_metadata(EapPacket::new(
            220,
            EapPacketData::Response {
                type_data: EapPacketTypeData::GenericTokenCard(vec![1, 2, 3]),
            },
        ));

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        assert_that!(container.has_metadata::<ChapPacketData>(), is_false());
    }

    #[test]
    fn should_not_add_chap_packet_data_metadata_to_container_when_there_is_no_eap_packet_metadata()
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

        let mut target = EapTypeMD5ChallengeRadiusInputPipelineStep::new();
        target.process(&mut container).unwrap();

        assert_that!(container.has_metadata::<ChapPacketData>(), is_false());
    }
}
