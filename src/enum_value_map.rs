#[macro_export]
macro_rules! enum_value_map {
    (enum $name:ident: $ty:ty { $($variant:ident <=> $value:literal,)* }) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum $name {
            $($variant,)*
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
