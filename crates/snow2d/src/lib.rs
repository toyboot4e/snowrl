/*!

Snow2D

A 2D framework on top of [`rokol`].

*/

pub use rokol;

pub mod asset;
pub mod gfx;

pub mod audio {
    //! `soloud-rs` re-exported

    pub use soloud::*;
}

pub mod input {
    //! `xdl` re-exported

    pub use xdl::*;
}
