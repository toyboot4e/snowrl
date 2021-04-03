/*!
Core utilities
*/

mod cheat;
pub use cheat::{cheat, Cheat};

pub mod ez;
pub mod pool;
pub mod tyobj;

/// [`arraytools::ArrayTools`] re-exported
///
///
pub use arraytools::ArrayTools;

pub mod arena {
    /*!
     Non-reference-counted pool

     This is a fork of [`thunderdome`]. Changes:

     * `Index` has type parameter `T`
     * `Arena::insert` accepts `impl Into<T>`
    */

    pub use thunderdome::*;
}

/// [`bytemuck`] re-exported
///
/// ---
pub use bytemuck;

/// [`delegate::delegate`] re-exported
///
/// ---
pub use delegate::delegate;

/// [`derivative::Derivative`] re-exported
///
/// ---
pub use derivative::Derivative;

pub use dyn_clone;

/// [`enum_dispatch::enum_dispatch`] re-exported
///
/// ---
pub use enum_dispatch::enum_dispatch;

/// [`hackfn::hackfn`] re-exported
///
/// ---
pub use hackfn::hackfn;

/// [`indoc::indoc`] re-exported
///
/// ---
pub use indoc::indoc;

/// [`inherent::inherent`] re-exported
///
/// ---
pub use inherent::inherent;

pub use once_cell;

/// [`trait_enum::trait_enum`] re-exported
///
/// ---
pub use trait_enum::trait_enum;

pub mod tweak {
    //! [inline_tweak] re-exported
    //!
    //! ```
    //! use snow2d::utils::tweak::*;
    //!
    //! let x = tweak!(1.0);
    //! ```

    pub use inline_tweak::{self, watch, Tweakable};

    /// Creates reloadable literal at runtime
    pub use inline_tweak::tweak;
}
