#[macro_export]
macro_rules! define_byte_enum {
    (
        $name:ident {
            $other_variant:ident = OTHER,
            $($variant:ident = $value:expr),* $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum $name {
            $( $variant, )*
            $other_variant(u8),
        }

        impl From<u8> for $name {
            fn from(byte: u8) -> Self {
                match byte {
                    $($value => $name::$variant),* ,
                    other => $name::$other_variant(other),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(byte_enum: $name) -> Self {
                match byte_enum {
                    $($name::$variant => $value),* ,
                    $name::$other_variant(other) => other,
                }
            }
        }
    };
    (
        $name:ident {
            $($variant:ident = $value:expr),* $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum $name {
            $( $variant, )*
        }

        impl TryFrom<u8> for $name {
            type Error = u8;

            fn try_from(byte: u8) -> Result<Self, Self::Error> {
                match byte {
                    $($value => Ok($name::$variant)),* ,
                    other => Err(other),
                }
            }
        }

        impl TryFrom<$name> for u8 {
            type Error = $name;

            fn try_from(byte_enum: $name) -> Result<Self, Self::Error> {
                match byte_enum {
                    $($name::$variant => Ok($value)),* ,
                    other => Err(other),
                }
            }
        }
    };
}
