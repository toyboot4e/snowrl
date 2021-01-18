/*!

[`soloud-rs`] re-exported with additional types and [`snow2d::asset`] integration

[`soloud-rs`]: https://github.com/MoAlyousef/soloud-rs
[`snow2d::asset`]: crate::asset

[SoLoud] is an easy to use, free, portable c/c++ audio engine for games.

[SoLoud]: https://sol.gfxile.net/soloud/

*/

pub use soloud::{audio as src, filter, prelude, Handle, Soloud as AudioDrop};

use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// Reference-counted ownership of [`AudioDrop`]
#[derive(Debug, Clone)]
pub struct Audio {
    /// I wanted to use `RefCell` but then [`Deref`] can't be implemented
    inner: Rc<AudioDrop>,
}

impl Audio {
    /// Two instances of [`AudioDrop`] can't exist
    pub unsafe fn create() -> Result<Self, prelude::SoloudError> {
        let inner = AudioDrop::default()?;
        Ok(Self {
            inner: Rc::new(inner),
        })
    }
}

// cheat the borrow checker..

impl Deref for Audio {
    type Target = AudioDrop;
    fn deref(&self) -> &Self::Target {
        unsafe { &*Rc::as_ptr(&self.inner) }
    }
}

impl DerefMut for Audio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(Rc::as_ptr(&self.inner) as *mut _) }
    }
}

pub mod asset {
    //! [`snow2d::asset`](crate::asset) integration

    use crate::{
        asset::{Asset, AssetCacheAny, AssetCacheT, AssetItem, AssetLoader},
        audio::{prelude::*, src, Audio, Handle},
    };

    use std::{fs, io};

    use std::path::Path;

    fn upcast_err<E>(e: E) -> std::io::Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }

    pub fn register_asset_loaders(assets: &mut AssetCacheAny, audio: &Audio) {
        assets.add_cache::<src::Wav>(AssetCacheT::new(WavLoader::new(audio.clone())));

        assets.add_cache::<src::WavStream>(AssetCacheT::new(WavStreamLoader::new(audio.clone())));
    }

    // ----------------------------------------
    // `src::Wav`
    impl AssetItem for src::Wav {
        type Loader = WavLoader;
    }

    /// [`AssetLoader`] of [`src::Wav`], which is for short audio files
    #[derive(Debug, Clone)]
    pub struct WavLoader {
        soloud: Audio,
    }

    impl WavLoader {
        pub fn new(soloud: Audio) -> Self {
            Self { soloud }
        }
    }

    impl AssetLoader for WavLoader {
        type Item = src::Wav;
        fn load(&mut self, path: &Path) -> io::Result<Self::Item> {
            let mem = fs::read(path)?;
            let item = Self::Item::from_mem(mem.into()).map_err(upcast_err)?;
            Ok(item)
        }
    }

    // ----------------------------------------
    // `src::WavStream`
    impl AssetItem for src::WavStream {
        type Loader = WavStreamLoader;
    }

    /// [`AssetLoader`] of [`src::WavStream`], which is for long audio files
    #[derive(Debug, Clone)]
    pub struct WavStreamLoader {
        soloud: Audio,
    }

    impl WavStreamLoader {
        pub fn new(soloud: Audio) -> Self {
            Self { soloud }
        }
    }

    impl AssetLoader for WavStreamLoader {
        type Item = src::WavStream;
        fn load(&mut self, path: &Path) -> io::Result<Self::Item> {
            let mem = fs::read(path)?;
            let item = Self::Item::from_mem(mem.into()).map_err(upcast_err)?;
            Ok(item)
        }
    }

    #[derive(Debug)]
    pub struct Playback {
        pub handle: Handle,
        pub song: Asset<src::WavStream>,
    }

    /// Storage to play one music
    #[derive(Debug)]
    pub struct MusicPlayer {
        pub audio: Audio,
        pub current: Option<Playback>,
    }

    impl MusicPlayer {
        pub fn new(audio: Audio) -> Self {
            Self {
                audio,
                current: None,
            }
        }

        pub fn play_song(&mut self, mut song: Asset<src::WavStream>) {
            if let Some(playback) = self.current.as_mut() {
                //
            }

            let handle = self.audio.play(&*song.get_mut().unwrap());
            self.current = Some(Playback { handle, song })
        }
    }
}
