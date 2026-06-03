use crate::connector::udp::config::UdpRadiusConnectorConfig;
use crate::connector::{RadiusConnector, RadiusConnectorError};
use async_trait::async_trait;
use modrad_radius::peer::RadiusPeer;
use modrad_radius::radius::RadiusPacket;
use modrad_radius::radius::container::{RadiusPacketInputContainer, RadiusPacketOutputContainer};
use std::net::SocketAddr;
use std::sync::Arc;
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
        let peer = Arc::new(RadiusPeer::Udp { remote_address });

        Ok(RadiusPacketInputContainer::new(packet, peer))
    }

    async fn send(
        &self,
        output_container: RadiusPacketOutputContainer,
    ) -> Result<(), RadiusConnectorError> {
        if let RadiusPeer::Udp { remote_address } = output_container.peer().as_ref() {
            let buffer = Vec::from(output_container.into_packet());
            self.socket.send_to(&buffer, remote_address).await?;
            return Ok(());
        }

        Err(RadiusConnectorError::InvalidPeer)
    }
}
