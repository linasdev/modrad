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
    Other(u8, Vec<u8>),
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
    Other(u8, Vec<u8>),
}

impl EapPacketData {
    pub fn length(&self) -> usize {
        match self {
            EapPacketData::Request { type_data } | EapPacketData::Response { type_data } => {
                1 + type_data.length()
            }
            EapPacketData::Success | EapPacketData::Failure => 0,
            EapPacketData::Other(_, buffer) => buffer.len(),
        }
    }

    pub fn code(&self) -> EapPacketCode {
        match self {
            EapPacketData::Request { .. } => EapPacketCode::Request,
            EapPacketData::Response { .. } => EapPacketCode::Response,
            EapPacketData::Success => EapPacketCode::Success,
            EapPacketData::Failure => EapPacketCode::Failure,
            EapPacketData::Other(code, _) => EapPacketCode::Other(*code),
        }
    }
}

impl From<EapPacketData> for Vec<u8> {
    fn from(data: EapPacketData) -> Self {
        match data {
            EapPacketData::Request { type_data } | EapPacketData::Response { type_data } => {
                let mut buffer = Vec::with_capacity(1 + type_data.length());
                buffer.push(type_data.packet_type().into());
                buffer.extend_from_slice(&Vec::from(type_data));
                buffer
            }
            EapPacketData::Success | EapPacketData::Failure => vec![],
            EapPacketData::Other(_, buffer) => buffer,
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
            _ => EapPacketData::Other(code.into(), buffer.to_vec()),
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

    pub fn packet_type(&self) -> EapPacketType {
        match self {
            EapPacketTypeData::Identity(_) => EapPacketType::Identity,
            EapPacketTypeData::Notification(_) => EapPacketType::Notification,
            EapPacketTypeData::Nak { .. } => EapPacketType::Nak,
            EapPacketTypeData::MD5Challenge(_) => EapPacketType::MD5Challenge,
            EapPacketTypeData::OneTimePassword(_) => EapPacketType::OneTimePassword,
            EapPacketTypeData::GenericTokenCard(_) => EapPacketType::GenericTokenCard,
            EapPacketTypeData::ExpandedType { .. } => EapPacketType::ExpandedType,
            EapPacketTypeData::Other(packet_type, _) => EapPacketType::Other(*packet_type),
        }
    }
}

impl From<EapPacketTypeData> for Vec<u8> {
    fn from(type_data: EapPacketTypeData) -> Self {
        match type_data {
            EapPacketTypeData::Identity(buffer) => buffer,
            EapPacketTypeData::Notification(buffer) => buffer,
            EapPacketTypeData::Nak { supported_types } => {
                supported_types.into_iter().map(u8::from).collect()
            }
            EapPacketTypeData::MD5Challenge(buffer) => buffer,
            EapPacketTypeData::OneTimePassword(buffer) => buffer,
            EapPacketTypeData::GenericTokenCard(buffer) => buffer,
            EapPacketTypeData::ExpandedType {
                vendor_id,
                vendor_type,
                vendor_data,
            } => {
                let mut buffer =
                    Vec::with_capacity(EAP_PACKET_EXPANDED_TYPE_HEADER_SIZE + vendor_data.len());

                for byte in &u32::to_be_bytes(vendor_id)[1..] {
                    buffer.push(*byte);
                }

                for byte in u32::to_be_bytes(vendor_type) {
                    buffer.push(byte);
                }

                buffer.extend_from_slice(&vendor_data);
                buffer
            }
            EapPacketTypeData::Other(_, buffer) => buffer,
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
            EapPacketType::Other(packet_type) => Ok(EapPacketTypeData::Other(packet_type, buffer.to_vec())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn should_convert_from_eap_packet_data_request_to_byte_buffer() {
        let data = EapPacketData::Request {
            type_data: EapPacketTypeData::Identity(vec![1, 2, 3]),
        };

        let result = Vec::from(data);

        let expected_result = vec![
            1, // type (Identity)
            1, 2, 3, // type data
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_data_response_to_byte_buffer() {
        let data = EapPacketData::Response {
            type_data: EapPacketTypeData::Identity(vec![1, 2, 3]),
        };

        let result = Vec::from(data);

        let expected_result = vec![
            1, // type (Identity)
            1, 2, 3, // type data
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_data_success_to_byte_buffer() {
        let data = EapPacketData::Success;
        let result = Vec::from(data);

        let expected_result = vec![];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_data_failure_to_byte_buffer() {
        let data = EapPacketData::Failure;
        let result = Vec::from(data);

        let expected_result = vec![];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_data_other_to_byte_buffer() {
        let data = EapPacketData::Other(0, vec![1, 2, 3]);
        let result = Vec::from(data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_eap_packet_data_request() {
        let code = EapPacketCode::Request;
        let buffer = vec![
            1, // type (Identity)
            1, 2, 3, // type data
        ];

        let result = EapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketData::Request {
                type_data: matches_pattern!(EapPacketTypeData::Identity(eq(&vec![1, 2, 3]))),
            })
        );
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_eap_packet_data_response() {
        let code = EapPacketCode::Response;
        let buffer = vec![
            1, // type (Identity)
            1, 2, 3, // type data
        ];

        let result = EapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketData::Response {
                type_data: matches_pattern!(EapPacketTypeData::Identity(eq(&vec![1, 2, 3]))),
            })
        );
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_eap_packet_data_success() {
        let code = EapPacketCode::Success;
        let buffer = vec![];

        let result = EapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(result, matches_pattern!(EapPacketData::Success),);
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_eap_packet_data_failure() {
        let code = EapPacketCode::Failure;
        let buffer = vec![];

        let result = EapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(result, matches_pattern!(EapPacketData::Failure),);
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_eap_packet_data_other() {
        let code = EapPacketCode::Other(0);
        let buffer = vec![1, 2, 3];

        let result = EapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketData::Other(
                eq(&0),
                eq(&[1, 2, 3])
            )),
        );
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_identity_to_byte_buffer() {
        let type_data = EapPacketTypeData::Identity(vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_notification_to_byte_buffer() {
        let type_data = EapPacketTypeData::Notification(vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_nak_to_byte_buffer() {
        let type_data = EapPacketTypeData::Nak {
            supported_types: vec![
                EapPacketType::MD5Challenge,
                EapPacketType::OneTimePassword,
                EapPacketType::GenericTokenCard,
            ],
        };

        let result = Vec::from(type_data);

        let expected_result = vec![
            4, // type (MD5Challenge)
            5, // type (OneTimePassword)
            6, // type (GenericTokenCard)
        ];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_md5_challenge_to_byte_buffer() {
        let type_data = EapPacketTypeData::MD5Challenge(vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_one_time_password_to_byte_buffer() {
        let type_data = EapPacketTypeData::OneTimePassword(vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_generic_token_card_to_byte_buffer() {
        let type_data = EapPacketTypeData::GenericTokenCard(vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_expanded_type_to_byte_buffer() {
        let type_data = EapPacketTypeData::ExpandedType {
            vendor_id: 1,
            vendor_type: 2,
            vendor_data: vec![3, 4, 5],
        };
        let result = Vec::from(type_data);

        let expected_result = vec![
            0, 0, 1, // vendor id
            0, 0, 0, 2, // vendor type
            3, 4, 5, // vendor data
        ];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_eap_packet_type_data_other_to_byte_buffer() {
        let type_data = EapPacketTypeData::Other(0, vec![1, 2, 3]);
        let result = Vec::from(type_data);

        let expected_result = vec![1, 2, 3];
        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_identity() {
        let packet_type = EapPacketType::Identity;
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::Identity(eq(&[1, 2, 3]))),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_notification() {
        let packet_type = EapPacketType::Notification;
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::Notification(eq(&[1, 2, 3]))),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_nak() {
        let packet_type = EapPacketType::Nak;
        let buffer = vec![
            4, // type (MD5Challenge)
            5, // type (OneTimePassword)
            6, // type (GenericTokenCard)
        ];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::Nak {
                supported_types: eq(&[
                    EapPacketType::MD5Challenge,
                    EapPacketType::OneTimePassword,
                    EapPacketType::GenericTokenCard,
                ])
            }),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_md5_challenge() {
        let packet_type = EapPacketType::MD5Challenge;
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::MD5Challenge(eq(&[1, 2, 3]))),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_one_time_password() {
        let packet_type = EapPacketType::OneTimePassword;
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::OneTimePassword(eq(&[1, 2, 3]))),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_generic_token_card() {
        let packet_type = EapPacketType::GenericTokenCard;
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::GenericTokenCard(eq(&[1, 2, 3]))),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_expanded_type() {
        let packet_type = EapPacketType::ExpandedType;
        let buffer = vec![
            0, 0, 1, // vendor id
            0, 0, 0, 2, // vendor type
            3, 4, 5, // vendor data
        ];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::ExpandedType {
                vendor_id: eq(&1),
                vendor_type: eq(&2),
                vendor_data: eq(&[3, 4, 5]),
            }),
        );
    }

    #[test]
    fn should_convert_from_type_and_byte_buffer_to_eap_packet_type_data_other() {
        let packet_type = EapPacketType::Other(0);
        let buffer = vec![1, 2, 3];

        let result = EapPacketTypeData::try_from((packet_type, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(EapPacketTypeData::Other(
                eq(&0),
                eq(&[1, 2, 3])
            )),
        );
    }
}
