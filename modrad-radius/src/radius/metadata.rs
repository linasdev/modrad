use std::any::Any;

pub trait RadiusPacketMetadata: Send {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
