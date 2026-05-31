use crate::packet::RadiusPacket;
use crate::packet::metadata::RadiusPacketMetadata;
use crate::peer::RadiusPeer;
use crate::pipeline::RadiusPipelineAcceptItem;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;

pub struct RadiusPacketInputContainer {
    packet: RadiusPacket,
    peer: RadiusPeer,
    metadata: HashMap<TypeId, Box<dyn RadiusPacketMetadata>>,
}

#[derive(Debug)]
pub struct RadiusPacketOutputContainer {
    packet: RadiusPacket,
    peer: RadiusPeer,
}

impl RadiusPacketInputContainer {
    pub fn new(packet: RadiusPacket, peer: RadiusPeer) -> Self {
        Self {
            packet,
            peer,
            metadata: HashMap::new(),
        }
    }

    pub fn packet(&self) -> &RadiusPacket {
        &self.packet
    }

    pub fn peer(&self) -> &RadiusPeer {
        &self.peer
    }

    pub fn remote_address(&self) -> &SocketAddr {
        match &self.peer {
            RadiusPeer::Udp { remote_address } => remote_address,
        }
    }

    pub fn has_metadata<T: RadiusPacketMetadata + 'static>(&self) -> bool {
        self.metadata.contains_key(&TypeId::of::<T>())
    }

    pub fn get_metadata<T: RadiusPacketMetadata + 'static>(&self) -> Option<&T> {
        self.metadata
            .get(&TypeId::of::<T>())
            .and_then(|value| value.as_any().downcast_ref::<T>())
    }

    pub fn set_metadata<T: RadiusPacketMetadata + 'static>(&mut self, value: T) -> Option<T> {
        self.metadata
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|old_value| old_value.into_any().downcast::<T>().ok())
            .map(|old_value| *old_value)
    }
}

impl RadiusPacketOutputContainer {
    pub fn new(packet: RadiusPacket, peer: RadiusPeer) -> Self {
        Self { packet, peer }
    }

    pub fn packet(&self) -> &RadiusPacket {
        &self.packet
    }

    pub fn packet_mut(&mut self) -> &mut RadiusPacket {
        &mut self.packet
    }

    pub fn peer(&self) -> &RadiusPeer {
        &self.peer
    }
}

impl Debug for RadiusPacketInputContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadiusPacketContainer")
            .field("packet", &self.packet)
            .field("peer", &self.peer)
            .field("metadata_count", &self.metadata.len())
            .finish()
    }
}

impl RadiusPipelineAcceptItem for RadiusPacketOutputContainer {}
