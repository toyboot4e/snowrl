/*!
Re-exports of glue code
*/

/// Utility for initializing the game window
pub type Init = rokol::glue::sdl::Init;

#[cfg(feature = "sdl2")]
/// Handles of platform-dependenct RAII objects
pub type Platform = rokol::glue::sdl::WindowHandle;
