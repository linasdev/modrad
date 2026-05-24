use crate::eap::packet::EapPacket;
use crate::packet::attribute::RadiusPacketAttributeType;
use crate::pipeline::container::RadiusPacketContainer;
use crate::pipeline::metadata::RadiusPacketMetadataKey;
use crate::pipeline::{RadiusPipeline, RadiusPipelineError};
use log::{debug, info};

#[derive(Default)]
pub struct EapRadiusPipeline {}

impl EapRadiusPipeline {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RadiusPipeline for EapRadiusPipeline {
    fn process(
        &mut self,
        packet_container: &mut RadiusPacketContainer,
    ) -> Result<(), RadiusPipelineError> {
        let attributes = packet_container.packet().attributes();

        if !attributes.has(RadiusPacketAttributeType::EAPMessage) {
            debug!("No EAP-Message attribute found in packet, skipping pipeline processing");
            return Ok(());
        }

        let eap_message: Vec<u8> = attributes
            .get(RadiusPacketAttributeType::EAPMessage)
            .into_iter()
            .flat_map(|attribute| attribute.value())
            .copied()
            .collect();

        let eap_packet = EapPacket::try_from(&eap_message[..])?;

        info!("Valid EAP-Message attribute found in packet, adding EapPacket metadata");

        packet_container.set_metadata(RadiusPacketMetadataKey::EapPacket, eap_packet);

        Ok(())
    }
}
