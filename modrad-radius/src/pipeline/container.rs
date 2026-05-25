use crate::packet::RadiusPacket;
use crate::peer::RadiusPeer;
use crate::pipeline::metadata::RadiusPacketMetadata;
use std::any::TypeId;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct RadiusPacketContainer {
    packet: RadiusPacket,
    peer: RadiusPeer,
    metadata: HashMap<TypeId, Box<dyn RadiusPacketMetadata>>,
}

impl RadiusPacketContainer {
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
