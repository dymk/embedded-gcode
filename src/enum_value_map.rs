#[macro_export]
macro_rules! enum_value_map {
    (enum $name:ident: $ty:ty { $($variant:ident <=> $value:literal,)* }) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum $name {
            $($variant,)*
        }

        impl crate::enum_value_map::EnumValueMap for $name {
            type Value = $ty;

            fn from_value(value: $ty) -> Option<Self> {
                match value {
                    $($value => Some(Self::$variant),)*
                    _ => None,
                }
            }
            fn to_value(&self) -> $ty {
                match self {
                    $(Self::$variant => $value,)*
                }
            }
        }
    };
}

pub trait EnumValueMap {
    type Value;

    fn from_value(value: Self::Value) -> Option<Self>
    where
        Self: Sized;
    fn to_value(&self) -> Self::Value;
}
