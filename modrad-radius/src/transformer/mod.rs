use crate::packet::RadiusPacket;
use crate::packet::container::RadiusPacketInputContainer;

pub mod message_authenticator;
pub mod response_authenticator;

#[derive(Debug)]
pub enum RadiusPacketTransformerError {}

pub trait RadiusPacketTransformer {
    fn transform(
        &mut self,
        packet: &mut RadiusPacket,
        original_packet_container: &RadiusPacketInputContainer,
    ) -> Result<(), RadiusPacketTransformerError>;
}
