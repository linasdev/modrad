use crate::eap::packet::EapPacketError;
use crate::eap::packet::code::EapPacketCode;
use modrad_macros::define_byte_enum;

const EAP_PACKET_EXPANDED_TYPE_HEADER_SIZE: usize = 7;

define_byte_enum!(
    EapPacketType {
        Other = OTHER,
        Identity = 1,
        Notification = 2,
        Nak = 3,
        MD5Challenge = 4,
        OneTimePassword = 5,
        GenericTokenCard = 6,
        ExpandedType = 254,
    }
);

#[derive(Debug)]
pub enum EapPacketData {
    Request { type_data: EapPacketTypeData },
    Response { type_data: EapPacketTypeData },
    Success,
    Failure,
    Other(EapPacketCode, Vec<u8>),
}

#[derive(Debug)]
pub enum EapPacketTypeData {
    Identity(Vec<u8>),
    Notification(Vec<u8>),
    Nak {
        supported_types: Vec<EapPacketType>,
    },
    MD5Challenge(Vec<u8>),
    OneTimePassword(Vec<u8>),
    GenericTokenCard(Vec<u8>),
    ExpandedType {
        vendor_id: u32,
        vendor_type: u32,
        vendor_data: Vec<u8>,
    },
    Other(EapPacketType, Vec<u8>),
}

impl EapPacketData {
    pub fn length(&self) -> usize {
        match self {
            EapPacketData::Request { type_data } | EapPacketData::Response { type_data } => 1 + type_data.length(),
            EapPacketData::Success | EapPacketData::Failure => 0,
            EapPacketData::Other(_, buffer) => buffer.len(),
        }
    }
}

impl TryFrom<(EapPacketCode, &[u8])> for EapPacketData {
    type Error = EapPacketError;

    fn try_from((code, buffer): (EapPacketCode, &[u8])) -> Result<Self, Self::Error> {
        let type_data = match code {
            EapPacketCode::Request | EapPacketCode::Response => {
                if buffer.len() < 1 {
                    return Err(EapPacketError::NotEnoughData);
                }

                let packet_type = EapPacketType::from(buffer[0]);
                let type_data = EapPacketTypeData::try_from((packet_type, &buffer[1..]))?;
                Some(type_data)
            }
            _ => None,
        };

        let data = match (code, type_data) {
            (EapPacketCode::Request, Some(type_data)) => EapPacketData::Request { type_data },
            (EapPacketCode::Response, Some(type_data)) => EapPacketData::Response { type_data },
            (EapPacketCode::Success, None) => EapPacketData::Success,
            (EapPacketCode::Failure, None) => EapPacketData::Failure,
            _ => EapPacketData::Other(code, buffer.to_vec()),
        };

        Ok(data)
    }
}

impl EapPacketTypeData {
    pub fn length(&self) -> usize {
        match self {
            EapPacketTypeData::Identity(buffer) => buffer.len(),
            EapPacketTypeData::Notification(buffer) => buffer.len(),
            EapPacketTypeData::Nak { supported_types } => supported_types.len(),
            EapPacketTypeData::MD5Challenge(buffer) => buffer.len(),
            EapPacketTypeData::OneTimePassword(buffer) => buffer.len(),
            EapPacketTypeData::GenericTokenCard(buffer) => buffer.len(),
            EapPacketTypeData::ExpandedType { vendor_data, .. } => {
                EAP_PACKET_EXPANDED_TYPE_HEADER_SIZE + vendor_data.len()
            }
            EapPacketTypeData::Other(_, buffer) => buffer.len(),
        }
    }
}

impl TryFrom<(EapPacketType, &[u8])> for EapPacketTypeData {
    type Error = EapPacketError;

    fn try_from((packet_type, buffer): (EapPacketType, &[u8])) -> Result<Self, Self::Error> {
        match packet_type {
            EapPacketType::Identity => Ok(EapPacketTypeData::Identity(buffer.to_vec())),
            EapPacketType::Notification => Ok(EapPacketTypeData::Notification(buffer.to_vec())),
            EapPacketType::Nak => {
                let supported_types = buffer.iter().copied().map(EapPacketType::from).collect();
                Ok(EapPacketTypeData::Nak { supported_types })
            }
            EapPacketType::MD5Challenge => Ok(EapPacketTypeData::MD5Challenge(buffer.to_vec())),
            EapPacketType::OneTimePassword => {
                Ok(EapPacketTypeData::OneTimePassword(buffer.to_vec()))
            }
            EapPacketType::GenericTokenCard => {
                Ok(EapPacketTypeData::GenericTokenCard(buffer.to_vec()))
            }
            EapPacketType::ExpandedType => {
                if buffer.len() < EAP_PACKET_EXPANDED_TYPE_HEADER_SIZE {
                    return Err(EapPacketError::NotEnoughData);
                }

                let vendor_id = u32::from_be_bytes([0, buffer[0], buffer[1], buffer[2]]);
                let vendor_type = u32::from_be_bytes([buffer[3], buffer[4], buffer[5], buffer[6]]);
                let vendor_data = buffer[EAP_PACKET_EXPANDED_TYPE_HEADER_SIZE..].to_vec();

                Ok(EapPacketTypeData::ExpandedType {
                    vendor_id,
                    vendor_type,
                    vendor_data,
                })
            }
            EapPacketType::Other(_) => Ok(EapPacketTypeData::Other(packet_type, buffer.to_vec())),
        }
    }
}
