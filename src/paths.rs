//! Automatically generated with `build.rs`

#![allow(unused)]
use snow2d::asset::AssetKey;
use std::{borrow::Cow, ffi::OsStr, path::Path};
const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
pub mod types {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    pub static ANIM_TYPES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("types/anim_types.ron"), None);
    pub mod actors {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static ACTOR_IMAGES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("types/actors/actor_images.ron"), None);
        pub static ACTOR_TYPES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("types/actors/actor_types.ron"), None);
        pub static ACTOR_PLACES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("types/actors/actor_places.ron"), None);
    }
}
pub static CHICKEN: &'static AssetKey<'static> = &AssetKey::new_const(as_path("chicken.png"), None);
pub mod img {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    pub mod kbd {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static KBD_2_PACK: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/kbd/kbd2-pack.json"), None);
        pub static KBD_2: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/kbd/kbd2.png"), None);
    }
    pub mod sourve {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static BALOON: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/sourve/baloon.png"), None);
        pub static B: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/sourve/b.png"), None);
        pub static A: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/sourve/a.png"), None);
    }
    pub mod title {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static CHOICES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/title/choices.png"), None);
        pub static SNOWRL: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/title/snowrl.png"), None);
    }
    pub mod effect {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static TKTK_FIRE_16: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/effect/tktk_Fire_16.png"), None);
    }
    pub mod pochi {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static WHAT: &'static AssetKey<'static> = &AssetKey::new_const(as_path("img/pochi/what.png"), None);
    }
}
pub mod map {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    pub mod ron {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    }
    pub mod images {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static META_TILES_IMG: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/images/meta-tiles-img.png"), None);
        pub mod nekura_1 {
            #![allow(unused)]
            use snow2d::asset::AssetKey;
            use std::{borrow::Cow, ffi::OsStr, path::Path};
            const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
            pub static M_MURA: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/images/nekura_1/m_mura.png"), None);
            pub static M_SNOW_02: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/images/nekura_1/m_snow02.png"), None);
        }
        pub mod denzi {
            #![allow(unused)]
            use snow2d::asset::AssetKey;
            use std::{borrow::Cow, ffi::OsStr, path::Path};
            const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
            pub static DUNGEON_1: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/images/denzi/dungeon_1.png"), None);
        }
    }
    pub mod tsx {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static META_TILES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tsx/meta-tiles.tsx"), None);
        pub static NEKURA_1: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tsx/nekura_1.tsx"), None);
        pub static NEKURA_SNOW: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tsx/nekura_snow.tsx"), None);
        pub static DENZI_1: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tsx/denzi_1.tsx"), None);
    }
    pub mod tmx {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static RL_START: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tmx/rl_start.tmx"), None);
        pub static RL_DUNGEON: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tmx/rl_dungeon.tmx"), None);
        pub static TILES: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tmx/tiles.tmx"), None);
        pub static TITLE: &'static AssetKey<'static> = &AssetKey::new_const(as_path("map/tmx/title.tmx"), None);
    }
}
pub static IKA_CHAN: &'static AssetKey<'static> = &AssetKey::new_const(as_path("ika-chan.png"), None);
pub mod scripts {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    pub static TEST: &'static AssetKey<'static> = &AssetKey::new_const(as_path("scripts/test.glsp"), None);
    pub static PLACE: &'static AssetKey<'static> = &AssetKey::new_const(as_path("scripts/place.glsp"), None);
}
pub mod fonts {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
}
pub mod sound {
    #![allow(unused)]
    use snow2d::asset::AssetKey;
    use std::{borrow::Cow, ffi::OsStr, path::Path};
    const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
    pub mod se {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static TALK_ENTER: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/talk_enter.wav"), None);
        pub static SWING: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/swing.wav"), None);
        pub static CURSOR: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/cursor.wav"), None);
        pub static TALK_LEAVE: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/talk_leave.wav"), None);
        pub static SELECT: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/select.wav"), None);
        pub static DEATH: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/death.wav"), None);
        pub static ATTACK: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/se/attack.wav"), None);
    }
    pub mod bgm {
        #![allow(unused)]
        use snow2d::asset::AssetKey;
        use std::{borrow::Cow, ffi::OsStr, path::Path};
        const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }
        pub static TW_041: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/bgm/tw041.mp3"), None);
        pub static FIELD_DARK_02: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/bgm/field dark02.mp3"), None);
        pub static FOREST_02: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/bgm/forest_02.mp3"), None);
        pub static X: &'static AssetKey<'static> = &AssetKey::new_const(as_path("sound/bgm/x.mp3"), None);
    }
}
