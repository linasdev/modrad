use crate::packet::RadiusPacket;
use crate::pipeline::metadata::RadiusPacketMetadata;
use std::any::TypeId;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct RadiusPacketContainer {
    packet: RadiusPacket,
    remote_address: SocketAddr,
    metadata: HashMap<TypeId, Box<dyn RadiusPacketMetadata>>,
}

impl RadiusPacketContainer {
    pub fn new(packet: RadiusPacket, remote_address: SocketAddr) -> Self {
        Self {
            packet,
            remote_address,
            metadata: HashMap::new(),
        }
    }

    pub fn packet(&self) -> &RadiusPacket {
        &self.packet
    }

    pub fn remote_address(&self) -> &SocketAddr {
        &self.remote_address
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
