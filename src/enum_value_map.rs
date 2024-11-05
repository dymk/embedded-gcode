#[macro_export]
macro_rules! enum_value_map {
    ($(#[$enum_meta:meta])? enum $name:ident: $ty:ty { $($(#[$variant_meta:meta])? $variant:ident <=> $value:literal,)* }) => {
        $(#[$enum_meta])?
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum $name {
            $($(#[$variant_meta])? $variant,)*
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant,)*];

            pub const fn from_value(value: $ty) -> Option<Self> {
                match value {
                    $($value => Some(Self::$variant),)*
                    _ => None,
                }
            }
            pub const fn to_value(self) -> $ty {
                match self {
                    $(Self::$variant => $value,)*
                }
            }
        }
    };
}
