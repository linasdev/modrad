use modrad_macros::define_byte_enum;

define_byte_enum!(
    ChapPacketCode {
        Challenge = 1,
        Response = 2,
        Success = 3,
        Failure = 4,
    }
);
