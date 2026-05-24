use crate::tag_length_value::TagLengthValue;
use modrad_macros::define_byte_enum;
use std::fmt::Debug;

pub type RadiusPacketAttribute = TagLengthValue<RadiusPacketAttributeType>;

pub struct RadiusPacketAttributes {
    attributes: Vec<RadiusPacketAttribute>,
}

define_byte_enum!(
    RadiusPacketAttributeType {
        Other = OTHER,
        UserName = 1,
        UserPassword = 2,
        CHAPPassword = 3,
        NASIPAddress = 4,
        NASPort = 5,
        ServiceType = 6,
        FramedProtocol = 7,
        FramedIPAddress = 8,
        FramedIPNetmask = 9,
        FramedRouting = 10,
        FilterId = 11,
        FramedMTU = 12,
        FramedCompression = 13,
        LoginIPHost = 14,
        LoginService = 15,
        LoginTCPPort = 16,
        // 17
        ReplyMessage = 18,
        CallbackNumber = 19,
        CallbackId = 20,
        // 21
        FramedRoute = 22,
        FramedIPXNetwork = 23,
        State = 24,
        Class = 25,
        VendorSpecific = 26,
        SessionTimeout = 27,
        IdleTimeout = 28,
        TerminationAction = 29,
        CalledStatiodId = 30,
        CallingStationId = 31,
        NASIdentifier = 32,
        ProxyState = 33,
        LoginLATService = 34,
        LoginLATNode = 35,
        LoginLATGroup = 36,
        FramedAppleTalkLink = 37,
        FramedAppleTalkNetwork = 38,
        FramedAppleTalkZone = 39,
        // 40 - 59
        CHAPChallenge = 60,
        NASPortType = 61,
        PortLimit = 62,
        LoginLATPort = 63,
        // 64 - 78
        EAPMessage = 79,
        MessageAuthenticator = 80,
    }
);

impl RadiusPacketAttributes {
    pub fn new() -> Self {
        Self { attributes: vec![] }
    }

    pub fn has(&self, attribute_type: RadiusPacketAttributeType) -> bool {
        self.attributes
            .iter()
            .any(|attribute| attribute.tag() == attribute_type)
    }

    pub fn count(&self, attribute_type: RadiusPacketAttributeType) -> usize {
        self.attributes
            .iter()
            .filter(|attribute| attribute.tag() == attribute_type)
            .count()
    }

    pub fn get(&self, attribute_type: RadiusPacketAttributeType) -> Vec<&RadiusPacketAttribute> {
        self.attributes
            .iter()
            .filter(|attribute| attribute.tag() == attribute_type)
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &RadiusPacketAttribute> {
        self.attributes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RadiusPacketAttribute> {
        self.attributes.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = RadiusPacketAttribute> {
        self.attributes.into_iter()
    }

    pub fn push(&mut self, attribute: RadiusPacketAttribute) {
        self.attributes.push(attribute)
    }

    pub fn remove(
        &mut self,
        attribute_type: RadiusPacketAttributeType,
        attribute_index: usize,
    ) -> Option<RadiusPacketAttribute> {
        let index = self
            .attributes
            .iter()
            .enumerate()
            .filter(|(_, attribute)| attribute.tag() == attribute_type)
            .map(|(index, _)| index)
            .nth(attribute_index);

        if let Some(index) = index {
            Some(self.attributes.remove(index))
        } else {
            None
        }
    }

    pub fn remove_all(
        &mut self,
        attribute_type: RadiusPacketAttributeType,
    ) -> Vec<RadiusPacketAttribute> {
        self.attributes
            .extract_if(.., |attribute| attribute.tag() != attribute_type)
            .collect()
    }
}

impl Debug for RadiusPacketAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_map = f.debug_map();

        for attribute in self.attributes.iter() {
            debug_map.entry(&attribute.tag(), &attribute.value());
        }

        debug_map.finish()
    }
}
