/*!
Audio module

That is, [`soloud-rs`] re-exported with additional types and [`asset`](../asset) integration.

[`soloud-rs`]: https://github.com/MoAlyousef/soloud-rs
---

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

    use std::{fmt, io, path::Path};

    use anyhow::*;

    use crate::{
        asset::{Asset, AssetCacheAny, AssetCacheT, AssetItem, AssetKey, AssetLoader},
        audio::{src, Audio, Handle},
    };

    /// Adds audio asset loaders to [`AssetCacheAny`]
    pub fn register_asset_loaders(assets: &mut AssetCacheAny, audio: &Audio) {
        reg::<src::Wav>(assets, audio.clone());
        reg::<src::WavStream>(assets, audio.clone());

        fn reg<T>(assets: &mut AssetCacheAny, audio: Audio)
        where
            T: crate::audio::prelude::FromExt + fmt::Debug + 'static,
        {
            assets.add_cache::<T>(AssetCacheT::new(AudioLoader {
                audio,
                _phantom: std::marker::PhantomData,
            }));
        }
    }

    /// [`AssetLoader`] for most of the audio source types
    #[derive(Debug)]
    pub struct AudioLoader<Src>
    where
        Src: crate::audio::prelude::FromExt + fmt::Debug + 'static,
    {
        audio: Audio,
        _phantom: std::marker::PhantomData<Src>,
    }

    impl<T> AssetItem for T
    where
        T: crate::audio::prelude::FromExt + fmt::Debug + 'static,
    {
        type Loader = AudioLoader<T>;
    }

    impl<T> AssetLoader for AudioLoader<T>
    where
        T: crate::audio::prelude::FromExt + fmt::Debug + 'static,
    {
        type Item = T;
        fn load(&mut self, path: &Path) -> io::Result<Self::Item> {
            Self::Item::from_path(path).map_err(self::upcast_err)
        }
    }

    fn upcast_err<E>(e: E) -> std::io::Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }

    /// Playback handle for [`MusicPlayer`]
    #[derive(Debug)]
    pub struct Playback {
        pub handle: Handle,
        pub song: Asset<src::WavStream>,
    }

    /// Background music player
    #[derive(Debug)]
    pub struct MusicPlayer {
        pub audio: Audio,
        /// [`Playback`] of current music
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
            if let Some(_playback) = self.current.as_mut() {
                // TODO: fade out
            }

            // TODO: fade in
            let handle = self.audio.play_background_ex(
                &*song.get_mut().unwrap(),
                1.0,
                false,
                Handle::PRIMARY,
            );

            self.current = Some(Playback { handle, song })
        }
    }

    impl AssetCacheT<src::Wav> {
        /// Play sound
        pub fn play<'a>(&mut self, sound: impl Into<AssetKey<'a>>, audio: &Audio) -> Result<()> {
            let mut se: Asset<src::Wav> = self.load_sync(sound)?;
            let se = se.get_mut().unwrap();
            audio.play(&*se);
            Ok(())
        }

        /// Play sound and set the preserve flag on the asset
        pub fn play_preserve<'a>(
            &mut self,
            sound: impl Into<AssetKey<'a>>,
            audio: &Audio,
        ) -> Result<()> {
            let mut se: Asset<src::Wav> = self.load_sync_preserve(sound)?;
            let se = se.get_mut().unwrap();
            audio.play(&*se);
            Ok(())
        }
    }

    impl AssetCacheAny {
        /// Play sound
        pub fn play<'a>(&mut self, sound: impl Into<AssetKey<'a>>, audio: &Audio) -> Result<()> {
            let cache: &mut AssetCacheT<src::Wav> = self
                .cache_mut()
                .ok_or_else(|| anyhow!("Unable to find cache of type WavStream"))?;
            cache.play(sound, audio)?;
            Ok(())
        }

        /// Play sound and set the preserve flag on the asset
        pub fn play_preserve<'a>(
            &mut self,
            sound: impl Into<AssetKey<'a>>,
            audio: &Audio,
        ) -> Result<()> {
            let cache: &mut AssetCacheT<src::Wav> = self
                .cache_mut()
                .ok_or_else(|| anyhow!("Unable to find cache of type WavStream"))?;
            cache.play_preserve(sound, audio)?;
            Ok(())
        }
    }
}
