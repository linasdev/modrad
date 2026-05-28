use crate::packet::RadiusPacket;
use crate::peer::RadiusPeer;

#[derive(Debug)]
pub enum RadiusHandlerAction<'p> {
    SendPacket {
        packet: RadiusPacket,
        peer: &'p RadiusPeer,
    },
    DiscardPacket,
    NextHandler,
}
