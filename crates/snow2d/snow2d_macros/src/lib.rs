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
