use crate::chap::ChapPacketError;
use crate::chap::code::ChapPacketCode;
use crate::radius::metadata::RadiusPacketMetadata;
use std::any::Any;

#[derive(Debug)]
pub enum ChapPacketData {
    Challenge { value: Vec<u8>, name: Vec<u8> },
    Response { value: Vec<u8>, name: Vec<u8> },
    Success { message: Vec<u8> },
    Failure { message: Vec<u8> },
}

impl ChapPacketData {
    pub fn length(&self) -> usize {
        match self {
            ChapPacketData::Challenge { value, name }
            | ChapPacketData::Response { value, name } => 1 + value.len() + name.len(),
            ChapPacketData::Success { message } | ChapPacketData::Failure { message } => {
                message.len()
            }
        }
    }

    pub fn code(&self) -> ChapPacketCode {
        match self {
            ChapPacketData::Challenge { .. } => ChapPacketCode::Challenge,
            ChapPacketData::Response { .. } => ChapPacketCode::Response,
            ChapPacketData::Success { .. } => ChapPacketCode::Success,
            ChapPacketData::Failure { .. } => ChapPacketCode::Failure,
        }
    }
}

impl From<ChapPacketData> for Vec<u8> {
    fn from(data: ChapPacketData) -> Self {
        let mut buffer = Vec::with_capacity(data.length());

        match data {
            ChapPacketData::Challenge { value, name }
            | ChapPacketData::Response { value, name } => {
                buffer.push(value.len() as u8);
                buffer.extend_from_slice(&value);
                buffer.extend_from_slice(&name);
                buffer
            }
            ChapPacketData::Success { message } | ChapPacketData::Failure { message } => {
                buffer.extend_from_slice(&message);
                buffer
            }
        }
    }
}

impl TryFrom<(ChapPacketCode, &[u8])> for ChapPacketData {
    type Error = ChapPacketError;

    fn try_from((packet_code, buffer): (ChapPacketCode, &[u8])) -> Result<Self, Self::Error> {
        match packet_code {
            ChapPacketCode::Challenge | ChapPacketCode::Response => {
                if buffer.len() < 3 {
                    return Err(ChapPacketError::NotEnoughData);
                }

                let value_length = buffer[0] as usize;

                if buffer.len() < value_length + 1 {
                    return Err(ChapPacketError::NotEnoughData);
                }

                let value = buffer[1..value_length + 1].to_vec();
                let name = buffer[value_length + 1..].to_vec();

                if packet_code == ChapPacketCode::Challenge {
                    Ok(ChapPacketData::Challenge { value, name })
                } else {
                    Ok(ChapPacketData::Response { value, name })
                }
            }
            ChapPacketCode::Success => Ok(ChapPacketData::Success {
                message: buffer.to_vec(),
            }),
            ChapPacketCode::Failure => Ok(ChapPacketData::Failure {
                message: buffer.to_vec(),
            }),
        }
    }
}

impl RadiusPacketMetadata for ChapPacketData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn should_convert_from_chap_packet_data_challenge_to_byte_buffer() {
        let data = ChapPacketData::Challenge {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        };

        let result = Vec::from(data);

        let expected_result = vec![
            3, // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_data_response_to_byte_buffer() {
        let data = ChapPacketData::Response {
            value: vec![1, 2, 3],
            name: vec![4, 5, 6],
        };

        let result = Vec::from(data);

        let expected_result = vec![
            3, // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_data_success_to_byte_buffer() {
        let data = ChapPacketData::Success {
            message: vec![1, 2, 3],
        };

        let result = Vec::from(data);

        let expected_result = vec![
            1, 2, 3, // message
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_chap_packet_data_failure_to_byte_buffer() {
        let data = ChapPacketData::Failure {
            message: vec![1, 2, 3],
        };

        let result = Vec::from(data);

        let expected_result = vec![
            1, 2, 3, // message
        ];

        assert_that!(result, eq(&expected_result));
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_chap_packet_data_challenge() {
        let code = ChapPacketCode::Challenge;
        let buffer = vec![
            3, // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        let result = ChapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacketData::Challenge {
                value: eq(&[1, 2, 3]),
                name: eq(&[4, 5, 6]),
            })
        )
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_chap_packet_data_response() {
        let code = ChapPacketCode::Response;
        let buffer = vec![
            3, // value length
            1, 2, 3, // value
            4, 5, 6, // name
        ];

        let result = ChapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacketData::Response {
                value: eq(&[1, 2, 3]),
                name: eq(&[4, 5, 6]),
            })
        )
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_chap_packet_data_success() {
        let code = ChapPacketCode::Success;
        let buffer = vec![
            1, 2, 3, // message
        ];

        let result = ChapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacketData::Success {
                message: eq(&[1, 2, 3]),
            })
        )
    }

    #[test]
    fn should_convert_from_code_and_byte_buffer_to_chap_packet_data_failure() {
        let code = ChapPacketCode::Failure;
        let buffer = vec![
            1, 2, 3, // message
        ];

        let result = ChapPacketData::try_from((code, &buffer[..])).unwrap();

        assert_that!(
            result,
            matches_pattern!(ChapPacketData::Failure {
                message: eq(&[1, 2, 3]),
            })
        )
    }

    #[test]
    fn should_not_convert_from_code_and_byte_buffer_to_chap_packet_data_challenge_when_there_is_not_enough_data()
     {
        let code = ChapPacketCode::Challenge;
        let buffer = vec![
            3, // value length
            1, 2, // missing byte
        ];

        let result = ChapPacketData::try_from((code, &buffer[..])).unwrap_err();

        assert_that!(result, matches_pattern!(ChapPacketError::NotEnoughData))
    }
}
