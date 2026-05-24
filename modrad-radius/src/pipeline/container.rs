use crate::packet::RadiusPacket;
use crate::pipeline::metadata::{RadiusPacketMetadata, RadiusPacketMetadataKey};
use std::collections::BTreeMap;
use std::net::SocketAddr;

pub struct RadiusPacketContainer {
    packet: RadiusPacket,
    remote_address: SocketAddr,
    metadata: BTreeMap<RadiusPacketMetadataKey, Box<dyn RadiusPacketMetadata>>,
}

impl RadiusPacketContainer {
    pub fn new(packet: RadiusPacket, remote_address: SocketAddr) -> Self {
        Self {
            packet,
            remote_address,
            metadata: BTreeMap::new(),
        }
    }

    pub fn packet(&self) -> &RadiusPacket {
        &self.packet
    }

    pub fn remote_address(&self) -> &SocketAddr {
        &self.remote_address
    }

    pub fn has_metadata_key(&self, metadata_key: &RadiusPacketMetadataKey) -> bool {
        self.metadata.contains_key(metadata_key)
    }

    pub fn get_metadata<T: RadiusPacketMetadata + 'static>(
        &self,
        metadata_key: &RadiusPacketMetadataKey,
    ) -> Option<&T> {
        self.metadata
            .get(metadata_key)
            .and_then(|value| value.as_any().downcast_ref::<T>())
    }

    pub fn set_metadata<T: RadiusPacketMetadata + 'static>(
        &mut self,
        metadata_key: RadiusPacketMetadataKey,
        value: T,
    ) -> Option<T> {
        self.metadata
            .insert(metadata_key, Box::new(value))
            .and_then(|old_value| old_value.into_any().downcast::<T>().ok())
            .map(|old_value| *old_value)
    }
}
