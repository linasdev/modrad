pub mod pool;
pub mod store;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Identifier {
    Radius(RadiusIdentifier),
    Eap(EapIdentifier),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RadiusIdentifier(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EapIdentifier(u8);

impl From<u8> for RadiusIdentifier {
    fn from(identifier: u8) -> Self {
        Self(identifier)
    }
}

impl From<RadiusIdentifier> for u8 {
    fn from(identifier: RadiusIdentifier) -> Self {
        identifier.0
    }
}

impl From<u8> for EapIdentifier {
    fn from(identifier: u8) -> Self {
        Self(identifier)
    }
}

impl From<EapIdentifier> for u8 {
    fn from(identifier: EapIdentifier) -> Self {
        identifier.0
    }
}
