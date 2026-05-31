use crate::connector::udp::config::UdpRadiusConnectorConfig;
use crate::connector::{RadiusConnector, RadiusConnectorError};
use async_trait::async_trait;
use modrad_radius::packet::RadiusPacket;
use modrad_radius::packet::container::RadiusPacketInputContainer;
use modrad_radius::peer::RadiusPeer;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub mod config;

pub struct UdpRadiusConnector {
    buffer_size: usize,
    socket: UdpSocket,
}

impl UdpRadiusConnector {
    pub async fn new(config: UdpRadiusConnectorConfig) -> Result<Self, RadiusConnectorError> {
        let socket_address = SocketAddr::new(config.address().parse()?, config.port());
        let socket = UdpSocket::bind(socket_address).await?;

        Ok(Self {
            buffer_size: config.buffer_size(),
            socket,
        })
    }
}

#[async_trait]
impl RadiusConnector for UdpRadiusConnector {
    async fn recv(&self) -> Result<RadiusPacketInputContainer, RadiusConnectorError> {
        let mut buffer = vec![0u8; self.buffer_size];
        let (length, remote_address) = self.socket.recv_from(&mut buffer).await?;
        let packet = RadiusPacket::try_from(&buffer[..length])?;
        let peer = RadiusPeer::Udp { remote_address };

        Ok(RadiusPacketInputContainer::new(packet, peer))
    }

    async fn send(
        &self,
        packet: RadiusPacket,
        peer: &RadiusPeer,
    ) -> Result<(), RadiusConnectorError> {
        if let RadiusPeer::Udp { remote_address } = peer {
            let buffer = Vec::from(packet);
            self.socket.send_to(&buffer, remote_address).await?;
            return Ok(());
        }

        Err(RadiusConnectorError::InvalidPeer)
    }
}
