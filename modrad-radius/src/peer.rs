use std::net::SocketAddr;

#[derive(Debug)]
pub enum RadiusPeer {
    Udp { remote_address: SocketAddr },
}
