//! [`xdl`] re-exported

pub use xdl::{utils, Dir4, Dir8, Input, Key, Keyboard, Sign};

pub mod vi {
    //! Virtual input

    pub use snow2d_macros::keys;
    pub use xdl::vi::*;
}
