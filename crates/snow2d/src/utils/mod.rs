/*!
Generic utilities
*/

pub mod ez;
pub mod pool;

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

/// [`enum_dispatch::enum_dispatch`] re-exported
///
/// ---
pub use enum_dispatch::enum_dispatch;

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