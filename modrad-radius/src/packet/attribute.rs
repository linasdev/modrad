use crate::tag_length_value::TagLengthValue;
use modrad_macros::define_byte_enum;

pub type RadiusPacketAttribute = TagLengthValue<RadiusPacketAttributeType>;

define_byte_enum!(
    RadiusPacketAttributeType {
        Other = OTHER,
        UserName = 1,
        UserPassword = 2,
    }
);
