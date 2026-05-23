use std::fmt::{Debug, Formatter};

const TAG_LENGTH_VALUE_HEADER_SIZE: usize = 2;

#[derive(Debug)]
pub enum TagLengthValueError {
    NotEnoughData,
    InvalidLength(usize),
}

#[derive(Clone, Eq, PartialEq)]
pub struct TagLengthValue<T> {
    tag: T,
    value: Vec<u8>,
}

impl<T> TagLengthValue<T>
where
    T: Copy,
{
    pub fn from_tag_and_value(tag: T, value: Vec<u8>) -> Self {
        Self { tag, value }
    }

    pub fn tag(&self) -> T {
        self.tag
    }

    pub fn length(&self) -> usize {
        TAG_LENGTH_VALUE_HEADER_SIZE + self.value.len()
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl<T> From<TagLengthValue<T>> for Vec<u8>
where
    T: Into<u8> + Copy,
{
    fn from(value: TagLengthValue<T>) -> Self {
        let mut buffer = Vec::with_capacity(value.length());

        buffer.push(value.tag.into()); // byte 0
        buffer.push(value.length() as u8); // byte 1
        buffer.extend_from_slice(&value.value); // bytes 2 - length

        buffer
    }
}

impl<T> TryFrom<&[u8]> for TagLengthValue<T>
where
    T: From<u8>,
{
    type Error = TagLengthValueError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < TAG_LENGTH_VALUE_HEADER_SIZE {
            return Err(TagLengthValueError::NotEnoughData);
        }

        let tag = T::from(buffer[0]);
        let length = buffer[1] as usize;

        if buffer.len() < length {
            return Err(TagLengthValueError::NotEnoughData);
        }

        if length < TAG_LENGTH_VALUE_HEADER_SIZE {
            return Err(TagLengthValueError::InvalidLength(length));
        }

        let value = buffer[TAG_LENGTH_VALUE_HEADER_SIZE..length].to_vec();

        Ok(Self { tag, value })
    }
}

impl<T> Debug for TagLengthValue<T>
where
    T: Debug + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagLengthValue")
            .field("tag", &self.tag)
            .field("length", &self.length())
            .finish()
    }
}
