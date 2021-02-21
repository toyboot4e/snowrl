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
    Fork of [`thunderdome`] re-exported. Note that [`Index`] is not reference-counted.
    */

    pub use thunderdome::*;
}

/// [`enum_dispatch::enum_dispatch`] re-exported
///
///
pub use enum_dispatch::enum_dispatch;

/// [`trait_enum::trait_enum`] re-exported
///
///
pub use trait_enum::trait_enum;

pub mod tweak {
    //! [inline_tweak] re-exported
    //!
    //! ```
    //! use rlbox::utils::tweak::*;
    //!
    //! let x = tweak!(1.0);
    //! ```

    pub use inline_tweak::{self, watch, Tweakable};

    /// Creates reloadable literal at runtime
    pub use inline_tweak::tweak;
}
