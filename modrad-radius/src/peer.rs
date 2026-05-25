use std::net::SocketAddr;

pub enum RadiusPeer {
    Udp { remote_address: SocketAddr },
}
