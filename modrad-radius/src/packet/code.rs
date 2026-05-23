#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RadiusPacketCode {
    AccessRequest,
    AccessAccept,
    AccessReject,
    AccountingRequest,
    AccountingResponse,
    AccessChallenge,
    Other(u8),
}

impl RadiusPacketCode {
    pub fn is_request(&self) -> bool {
        matches!(
            self,
            RadiusPacketCode::AccessRequest | RadiusPacketCode::AccountingRequest
        )
    }

    pub fn is_response(&self) -> bool {
        matches!(
            self,
            RadiusPacketCode::AccessAccept
                | RadiusPacketCode::AccessReject
                | RadiusPacketCode::AccessChallenge
                | RadiusPacketCode::AccountingResponse
        )
    }
}

impl From<u8> for RadiusPacketCode {
    fn from(value: u8) -> Self {
        match value {
            1 => RadiusPacketCode::AccessRequest,
            2 => RadiusPacketCode::AccessAccept,
            3 => RadiusPacketCode::AccessReject,
            4 => RadiusPacketCode::AccountingRequest,
            5 => RadiusPacketCode::AccountingResponse,
            11 => RadiusPacketCode::AccessChallenge,
            _ => RadiusPacketCode::Other(value),
        }
    }
}
