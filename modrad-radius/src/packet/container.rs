use crate::packet::RadiusPacket;
use crate::packet::metadata::RadiusPacketMetadata;
use crate::peer::RadiusPeer;
use crate::pipeline::RadiusPipelineAcceptItem;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct RadiusPacketContainer {
    packet: RadiusPacket,
    peer: Arc<RadiusPeer>,
    metadata: HashMap<TypeId, Box<dyn RadiusPacketMetadata>>,
}

#[derive(Debug)]
pub struct RadiusPacketInputContainer(RadiusPacketContainer);

#[derive(Debug)]
pub struct RadiusPacketOutputContainer(RadiusPacketContainer);

impl RadiusPacketContainer {
    pub fn new(packet: RadiusPacket, peer: Arc<RadiusPeer>) -> Self {
        Self {
            packet,
            peer,
            metadata: HashMap::new(),
        }
    }

    pub fn packet(&self) -> &RadiusPacket {
        &self.packet
    }

    pub fn peer(&self) -> Arc<RadiusPeer> {
        self.peer.clone()
    }

    pub fn remote_address(&self) -> &SocketAddr {
        match self.peer.as_ref() {
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

impl RadiusPacketInputContainer {
    pub fn new(packet: RadiusPacket, peer: Arc<RadiusPeer>) -> Self {
        Self(RadiusPacketContainer::new(packet, peer))
    }
}

impl Deref for RadiusPacketInputContainer {
    type Target = RadiusPacketContainer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RadiusPacketInputContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RadiusPacketOutputContainer {
    pub fn new(packet: RadiusPacket, peer: Arc<RadiusPeer>) -> Self {
        Self(RadiusPacketContainer::new(packet, peer))
    }

    pub fn packet_mut(&mut self) -> &mut RadiusPacket {
        &mut self.0.packet
    }

    pub fn into_packet(self) -> RadiusPacket {
        self.0.packet
    }
}

impl Deref for RadiusPacketOutputContainer {
    type Target = RadiusPacketContainer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RadiusPacketOutputContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for RadiusPacketContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadiusPacketContainer")
            .field("packet", &self.packet)
            .field("peer", &self.peer)
            .field("metadata_count", &self.metadata.len())
            .finish()
    }
}

impl RadiusPipelineAcceptItem for RadiusPacketOutputContainer {}
