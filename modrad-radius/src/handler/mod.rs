use crate::handler::action::RadiusHandlerAction;
use crate::pipeline::container::RadiusPacketContainer;

pub mod action;
pub mod message_authenticator;

#[derive(Debug)]
pub enum RadiusHandlerError {
}

pub trait RadiusHandler {
    fn handle<'p>(
        &mut self,
        packet_container: &'p RadiusPacketContainer,
    ) -> Result<RadiusHandlerAction<'p>, RadiusHandlerError>;
}
