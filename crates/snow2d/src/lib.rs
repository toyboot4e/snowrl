/*!

Snow2D

A 2D framework on top of [`rokol`].

*/

pub extern crate rokol;

pub mod asset;
pub mod audio;
pub mod gfx;

pub mod input {
    //! `xdl` re-exported

    pub use xdl::*;
}
