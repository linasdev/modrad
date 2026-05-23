use crate::tag_length_value::TagLengthValue;
use modrad_macros::define_byte_enum;

pub type RadiusPacketAttribute = TagLengthValue<RadiusPacketAttributeType>;

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
    }
);
