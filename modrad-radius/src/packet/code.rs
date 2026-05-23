use modrad_macros::define_byte_enum;

define_byte_enum!(
    RadiusPacketCode {
        Other = OTHER,
        AccessRequest = 1,
        AccessAccept = 2,
        AccessReject = 3,
        AccountingRequest = 4,
        AccountingResponse = 5,
        AccessChallenge = 11,
    }
);

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
