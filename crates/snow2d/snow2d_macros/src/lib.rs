//! Just for placing macros in somewhere other than crate root

/// TODO: handle C-K or such that
#[macro_export]
macro_rules! keys {
    [ $( $d:expr ),* ] => {
        vec![
            $($d.into(),)*
        ]
    }
}

/// Implements `From` and `Into` using `SerdeRepr` method
#[macro_export]
macro_rules! connect_repr_target {
    // T: TypeObject, U: From<TypeObject>
    ($T:ty, $U:ty) => {
        // SerdeRepr<TypeObject> -> Target
        impl From<snow2d::utils::tyobj::SerdeRepr<$T>> for $U {
            fn from(repr: snow2d::utils::tyobj::SerdeRepr<$T>) -> $U {
                <$U as snow2d::utils::tyobj::SerdeViaTyObj>::from_tyobj_repr(repr)
            }
        }

        // Target -> SerdeRepr<TypeObject>
        impl Into<snow2d::utils::tyobj::SerdeRepr<$T>> for $U {
            fn into(self: $U) -> snow2d::utils::tyobj::SerdeRepr<$T> {
                <$U as snow2d::utils::tyobj::SerdeViaTyObj>::into_tyobj_repr(self)
            }
        }
    };
}
