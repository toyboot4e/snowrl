/*
SnowRL configuration
*/

use snow2d::utils::Inspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum ShadowConfig {
    Blur,
    Raw,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum SnowConfig {
    Blizzard,
    // Light,
    None,
}

#[derive(Debug, Clone, Inspect)]
pub struct GameConfig {
    /// Global sound volume
    pub vol: f32,
    pub shadow_cfg: ShadowConfig,
    pub snow_cfg: SnowConfig,
}

impl GameConfig {
    //
}
