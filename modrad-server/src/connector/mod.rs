use async_trait::async_trait;
use modrad_radius::radius::RadiusPacketError;
use modrad_radius::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use std::net::AddrParseError;
use tokio::io;

pub mod udp;

#[derive(Debug)]
pub enum RadiusConnectorError {
    InvalidPeer,
    RadiusPacket(RadiusPacketError),
    AddrParse(AddrParseError),
    Io(io::Error),
}

#[async_trait]
pub trait RadiusConnector {
    async fn recv(&self) -> Result<RadiusPacketInputContainer, RadiusConnectorError>;
    async fn send(
        &self,
        output_container: RadiusPacketOutputContainer,
    ) -> Result<(), RadiusConnectorError>;
}

impl From<RadiusPacketError> for RadiusConnectorError {
    fn from(error: RadiusPacketError) -> Self {
        RadiusConnectorError::RadiusPacket(error)
    }
}

impl From<AddrParseError> for RadiusConnectorError {
    fn from(error: AddrParseError) -> Self {
        RadiusConnectorError::AddrParse(error)
    }
}

impl From<io::Error> for RadiusConnectorError {
    fn from(error: io::Error) -> Self {
        RadiusConnectorError::Io(error)
    }
}
