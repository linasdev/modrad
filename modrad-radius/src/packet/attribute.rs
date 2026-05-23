use crate::tag_length_value::TagLengthValue;

pub type RadiusPacketAttribute = TagLengthValue<RadiusPacketAttributeType>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RadiusPacketAttributeType {
    UserName,
    UserPassword,
    Other(u8),
}

impl From<u8> for RadiusPacketAttributeType {
    fn from(value: u8) -> Self {
        match value {
            1 => RadiusPacketAttributeType::UserName,
            2 => RadiusPacketAttributeType::UserPassword,
            _ => RadiusPacketAttributeType::Other(value),
        }
    }
}
