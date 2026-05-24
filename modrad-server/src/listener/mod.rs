use async_trait::async_trait;
use modrad_radius::packet::RadiusPacketError;
use modrad_radius::pipeline::container::RadiusPacketContainer;
use std::net::AddrParseError;
use tokio::io;

pub mod udp;

#[derive(Debug)]
pub enum RadiusListenerError {
    RadiusPacket(RadiusPacketError),
    AddrParse(AddrParseError),
    Io(io::Error),
}

#[async_trait]
pub trait RadiusListener {
    async fn recv(&self) -> Result<RadiusPacketContainer, RadiusListenerError>;
}

impl From<RadiusPacketError> for RadiusListenerError {
    fn from(error: RadiusPacketError) -> Self {
        RadiusListenerError::RadiusPacket(error)
    }
}

impl From<AddrParseError> for RadiusListenerError {
    fn from(error: AddrParseError) -> Self {
        RadiusListenerError::AddrParse(error)
    }
}

impl From<io::Error> for RadiusListenerError {
    fn from(error: io::Error) -> Self {
        RadiusListenerError::Io(error)
    }
}
