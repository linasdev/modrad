use crate::packet::RadiusPacket;
use crate::packet::container::RadiusPacketContainer;

pub mod message_authenticator;
pub mod response_authenticator;

#[derive(Debug)]
pub enum RadiusPacketTransformerError {}

pub trait RadiusPacketTransformer {
    fn transform(
        &mut self,
        packet: &mut RadiusPacket,
        original_packet_container: &RadiusPacketContainer,
    ) -> Result<(), RadiusPacketTransformerError>;
}
