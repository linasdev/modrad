use modrad_macros::define_byte_enum;

define_byte_enum!(
    EapPacketCode {
        Other = OTHER,
        Request = 1,
        Response = 2,
        Success = 3,
        Failure = 4,
    }
);
