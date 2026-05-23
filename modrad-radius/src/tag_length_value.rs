const TAG_LENGTH_VALUE_HEADER_SIZE: usize = 2;

#[derive(Debug)]
pub enum TagLengthValueError {
    NotEnoughData,
    InvalidLength(usize),
}

pub struct TagLengthValue<T>
where
    T: From<u8> + Copy,
{
    tag: T,
    length: usize,
    value: Vec<u8>,
}

impl<T> TagLengthValue<T>
where
    T: From<u8> + Copy,
{
    pub fn tag(&self) -> T {
        self.tag
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl<T> TryFrom<&[u8]> for TagLengthValue<T>
where
    T: From<u8> + Copy,
{
    type Error = TagLengthValueError;

    fn try_from(packet_data: &[u8]) -> Result<Self, Self::Error> {
        if packet_data.len() < TAG_LENGTH_VALUE_HEADER_SIZE {
            return Err(TagLengthValueError::NotEnoughData);
        }

        let tag = T::from(packet_data[0]);
        let length = packet_data[1] as usize;

        if packet_data.len() < length {
            return Err(TagLengthValueError::NotEnoughData);
        }

        if length < TAG_LENGTH_VALUE_HEADER_SIZE {
            return Err(TagLengthValueError::InvalidLength(length));
        }

        let value = packet_data[TAG_LENGTH_VALUE_HEADER_SIZE..length].to_vec();

        Ok(Self { tag, length, value })
    }
}
