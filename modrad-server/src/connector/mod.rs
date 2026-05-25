use async_trait::async_trait;
use modrad_radius::packet::{RadiusPacket, RadiusPacketError};
use modrad_radius::peer::RadiusPeer;
use modrad_radius::pipeline::container::RadiusPacketContainer;
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
    async fn recv(&self) -> Result<RadiusPacketContainer, RadiusConnectorError>;
    async fn send(
        &self,
        packet: RadiusPacket,
        peer: &RadiusPeer,
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
