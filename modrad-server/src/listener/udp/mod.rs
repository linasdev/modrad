use crate::listener::udp::config::UdpRadiusListenerConfig;
use crate::listener::{RadiusListener, RadiusListenerError};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use modrad_radius::packet::radius::RadiusPacket;

pub mod config;

pub struct UdpRadiusListener {
    buffer_size: usize,
    socket: UdpSocket,
}

impl UdpRadiusListener {
    pub async fn new(config: UdpRadiusListenerConfig) -> Result<Self, RadiusListenerError> {
        let socket_address = SocketAddr::new(config.address().parse()?, config.port());
        let socket = UdpSocket::bind(socket_address).await?;

        Ok(Self {
            buffer_size: config.buffer_size(),
            socket,
        })
    }
}

#[async_trait]
impl RadiusListener for UdpRadiusListener {
    async fn recv(&self) -> Result<(RadiusPacket, SocketAddr), RadiusListenerError> {
        let mut buffer = vec![0u8; self.buffer_size];
        let (length, remote_address) = self.socket.recv_from(&mut buffer).await?;
        let packet = RadiusPacket::try_from(&buffer[..length])?;

        Ok((packet, remote_address))
    }
}
