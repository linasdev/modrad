use std::any::Any;

pub trait RadiusPacketMetadata {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RadiusPacketMetadataKey {
    EapPacket,
    UserIdentifier,
    UserCredential,
}
