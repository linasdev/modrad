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

        $crate::paste! {
            #[cfg(test)]
            mod [<$name:snake:lower _tests>] {
                use super::*;
                use googletest::prelude::*;

                $(
                    #[test]
                    fn [<should_convert_from_u8_to_ $name:snake:lower _ $variant:snake:lower>]() {
                        assert_that!($name::from($value), eq($name::$variant))
                    }
                )*

                $(
                    #[test]
                    fn [<should_convert_from_ $name:snake:lower _ $variant:snake:lower _to_u8>]() {
                        assert_that!(u8::from($name::$variant), eq($value));
                    }
                )*

                #[test]
                fn [<should_convert_from_u8_to_ $name:snake:lower _ $other_variant:snake:lower>]() {
                    assert_that!($name::from(0), eq($name::$other_variant(0)));
                }

                #[test]
                fn [<should_convert_from_ $name:snake:lower _ $other_variant:snake:lower _to_u8>]() {
                    assert_that!(u8::from($name::$other_variant(0)), eq(0));
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

        impl From<$name> for u8 {
            fn from(byte_enum: $name) -> Self {
                match byte_enum {
                    $($name::$variant => $value),* ,
                }
            }
        }

        $crate::paste! {
            #[cfg(test)]
            mod [<$name:snake:lower _tests>] {
                use super::*;
                use googletest::prelude::*;

                $(
                    #[test]
                    fn [<should_convert_from_u8_to_ $name:snake:lower _ $variant:snake:lower>]() {
                        assert_that!($name::try_from($value), eq(Ok($name::$variant)));
                    }
                )*

                $(
                    #[test]
                    fn [<should_convert_from_ $name:snake:lower _ $variant:snake:lower _to_u8>]() {
                        assert_that!(u8::from($name::$variant), eq($value));
                    }
                )*

                #[test]
                fn [<should_not_convert_from_u8_to_ $name:snake:lower>]() {
                    assert_that!($name::try_from(0), eq(Err(0)));
                }
            }
        }
    };
}
