/*!
Asset cache and reference-counted asset references

# Serde support

1. Asset data should be serialized as `PathBuf`.
2. Asset handles should be serialized without creating duplicates (a.k.a. intering).

# TODOs

* Fix
    * release cache on no owner
    * do not allocate PathBuf

* Features
    * weak pointer?
    * serde (interning `Arc` asset handles)
    * async loading
    * dynamic loading
*/

#![allow(dead_code)]

/// `std::io::Result` re-exported
///
/// ---
pub use std::io::Result;

use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashMap,
    fmt, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use downcast_rs::{impl_downcast, Downcast};
use once_cell::sync::OnceCell;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

use crate::utils::cheat::Cheat;

/// Get asset path relative to `assets` directory
pub fn path(path: impl AsRef<Path>) -> PathBuf {
    // TODO: supply appropreate root path
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets = PathBuf::from(root).join("assets");
    assets.join(path)
}

pub fn deserialize_ron<'a, T: serde::de::DeserializeOwned>(
    key: impl Into<AssetKey<'a>>,
) -> anyhow::Result<T> {
    use std::fs;

    let path = path(key.into().deref());
    log::trace!("deserializing `{}`", path.display());
    let s = fs::read_to_string(&path)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("Unable to read asset file at `{}`", path.display()))?;
    ron::de::from_str::<T>(&s)
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "Unable to create asset from file `{}` for type {}",
                path.display(),
                std::any::type_name::<T>()
            )
        })
}

/// Asset data
pub trait AssetItem: fmt::Debug + Sized + 'static {
    type Loader: AssetLoader<Item = Self>;
}

/// How to load an [`AssetItem`]
pub trait AssetLoader: fmt::Debug + Sized + 'static {
    type Item: AssetItem;
    fn load(&mut self, path: &Path) -> Result<Self::Item>;
}

/// Shared ownership of an [`AssetItem`]
#[derive(Debug)]
pub struct Asset<T: AssetItem> {
    item: Option<Arc<Mutex<T>>>,
    path: Rc<PathBuf>,
}

// TODO: impl deref with dummy asset

impl<T: AssetItem> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.as_ref().map(|x| Arc::clone(x)),
            path: Rc::clone(&self.path),
        }
    }
}

impl<T: AssetItem> Asset<T> {
    pub fn is_loaded(&self) -> bool {
        self.item.is_some()
    }

    /// Tries to get `&T`, fails if the asset is not loaded or failed to load
    ///
    /// This step is for asynchrounous loading and hot reloaidng.
    ///
    /// Unfortunatelly, the return type is not `Option<&T>` and doesn't implement trait for type `T`.
    /// Still, you can use `&*asset.get()` to cast it to `&T`.
    pub fn get<'a>(&'a self) -> Option<impl Deref<Target = T> + 'a> {
        self.item.as_ref()?.lock().ok()
    }

    /// Tries to get `&mut T`, fails if the asset is not loaded or panics ([`Mutex`] under the hood)
    ///
    /// This step is for asynchrounous loading and hot reloaidng.
    ///
    /// Unfortunatelly, the return type is not `Option<&mut T>` and doesn't implement trait for type
    /// `T`. Still, you can use `&mut *asset.get()` to cast it to `&mut T`.
    pub fn get_mut<'a>(&'a mut self) -> Option<impl DerefMut + Deref<Target = T> + 'a> {
        self.item.as_mut()?.lock().ok()
    }
}

/// Key to load asset
///
/// TODO: use newtype struct while enabling static construction (or use IntoAssetKey)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "PathBuf")]
#[serde(into = "PathBuf")]
pub struct AssetKey<'a>(Cow<'a, Path>);

// TODO: interning on serde

impl<'a> From<PathBuf> for AssetKey<'a> {
    fn from(p: PathBuf) -> Self {
        Self(Cow::from(p))
    }
}

impl<'a> Into<PathBuf> for AssetKey<'a> {
    fn into(self) -> PathBuf {
        self.0.into_owned()
    }
}

impl<'a> AssetKey<'a> {
    pub fn new(p: impl Into<Cow<'a, Path>>) -> Self {
        AssetKey(p.into())
    }
}

impl<'a> std::ops::Deref for AssetKey<'a> {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Key to load asset (allocated statically)
#[derive(Clone, Copy)]
pub struct StaticAssetKey(pub &'static str);

impl<'a> Into<AssetKey<'a>> for StaticAssetKey {
    fn into(self) -> AssetKey<'a> {
        AssetKey(Cow::Borrowed(self.0.as_ref()))
    }
}

impl AsRef<Path> for StaticAssetKey {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

/// Newtype of [`PathBuf`], which is relative path from "root" asset directory
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetId {
    identity: PathBuf,
}

impl AssetId {
    pub fn from_key(key: &AssetKey) -> Self {
        Self {
            identity: key.to_path_buf(),
        }
    }
}

/// Access to an [`AssetItem`] with metadata
#[derive(Debug)]
struct AssetCacheEntry<T: AssetItem> {
    id: AssetId,
    path: Rc<PathBuf>,
    asset: Asset<T>,
}

/// Cache of a specific [`AssetItem`] type
#[derive(Debug)]
pub struct AssetCacheT<T: AssetItem> {
    entries: Vec<AssetCacheEntry<T>>,
    loader: T::Loader,
}

impl<T: AssetItem> AssetCacheT<T> {
    pub fn new(loader: T::Loader) -> Self {
        Self {
            entries: Vec::with_capacity(16),
            loader,
        }
    }

    pub fn load_sync<'a>(&mut self, key: impl Into<AssetKey<'a>>) -> Result<Asset<T>> {
        let key = key.into();
        let id = AssetId::from_key(&key);
        if let Some(a) = self.search_cache(&id) {
            log::trace!("(cache found for `{}` of type `{}`)", key.display(),  std::any::type_name::<T>());
            Ok(a)
        } else {
            log::debug!("loading asset `{}` of type `{}`", key.display(), std::any::type_name::<T>());
            self.load_new_sync(id)
        }
    }

    fn search_cache(&mut self, id: &AssetId) -> Option<Asset<T>> {
        self.entries
            .iter()
            .find(|a| a.id == *id)
            .map(|a| a.asset.clone())
    }

    fn load_new_sync(&mut self, id: AssetId) -> Result<Asset<T>> {
        let path = Rc::new(self::path(&id.identity));

        let asset = Asset {
            item: {
                let item = self.loader.load(&path)?;
                Some(Arc::new(Mutex::new(item)))
            },
            path: Rc::clone(&path),
        };

        let entry = AssetCacheEntry {
            id,
            path: Rc::clone(&path),
            asset: asset.clone(),
        };
        self.entries.push(entry);

        Ok(asset)
    }
}

/// Upcast of [`AssetCacheT`]
trait FreeUnused: fmt::Debug + Downcast {
    fn free_unused(&mut self);
}

impl_downcast!(FreeUnused);

impl<T: AssetItem> FreeUnused for AssetCacheT<T> {
    fn free_unused(&mut self) {
        let mut i = 0;
        let mut len = self.entries.len();
        while i < len {
            if let Some(item) = &mut self.entries[i].asset.item {
                if Arc::strong_count(item) == 1 {
                    log::debug!(
                        "free asset with path `{}` in slot `{}` of cache for type `{}`",
                        self.entries[i].path.display(),
                        i,
                        std::any::type_name::<T>(),
                    );
                    self.entries.remove(i);
                    len -= 1;
                }
            }
            i += 1;
        }
    }
}

/// Cache of any [`AssetItem`] type, a bundle of [`AssetCacheT`]s
#[derive(Debug)]
pub struct AssetCacheAny {
    caches: HashMap<TypeId, Box<dyn FreeUnused>>,
}

impl AssetCacheAny {
    pub fn new() -> Self {
        Self {
            caches: HashMap::with_capacity(16),
        }
    }

    pub fn free_unused(&mut self) {
        for cache in &mut self.caches.values_mut() {
            cache.free_unused();
        }
    }

    pub fn add_cache<T: AssetItem>(&mut self, cache: AssetCacheT<T>) {
        self.caches.insert(TypeId::of::<T>(), Box::new(cache));
    }

    pub fn cache_mut<T: AssetItem>(&mut self) -> Option<&mut AssetCacheT<T>> {
        let boxed = self.caches.get_mut(&TypeId::of::<T>()).unwrap();
        boxed.downcast_mut::<AssetCacheT<T>>()
    }

    pub fn load_sync<'a, T: AssetItem>(
        &mut self,
        key: impl Into<AssetKey<'a>>,
    ) -> Result<Asset<T>> {
        let key = key.into();
        self.cache_mut::<T>()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Non-existing asset cache for type {}",
                        std::any::type_name::<T>()
                    ),
                )
            })?
            .load_sync(key)
    }
}

/// Deserialize assets without making duplicates using thread-local variable
#[derive(Debug)]
pub struct AssetDeState {
    cache: Cheat<AssetCacheAny>,
}

static mut DE_STATE: OnceCell<AssetDeState> = OnceCell::new();

impl AssetDeState {
    /// Make sure the memory location of `cache` doesn't change until we call `end`
    pub unsafe fn start(cache: &mut AssetCacheAny) -> std::result::Result<(), Self> {
        DE_STATE.set(Self {
            cache: Cheat::new(cache),
        })
    }

    pub unsafe fn end() -> std::result::Result<(), ()> {
        match DE_STATE.take() {
            Some(_) => Ok(()),
            None => Err(()),
        }
    }

    pub unsafe fn cache_mut() -> Option<Cheat<AssetCacheAny>> {
        DE_STATE.get_mut().map(|me| Cheat::clone(&me.cache))
    }
}

impl<T: AssetItem> Serialize for Asset<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // serialize as PathBuf
        self.path.serialize(serializer)
    }
}

// TODO: Ensure to not panic while deserializing
impl<'de, T: AssetItem> Deserialize<'de> for Asset<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // deserialize as PathBuf
        let path = <PathBuf as Deserialize>::deserialize(deserializer)
            .map_err(|e| format!("Unable to load asset as `PathBuf`: {}", e))
            .unwrap();

        // load asset
        let state = unsafe {
            DE_STATE
                .get_mut()
                .ok_or_else(|| "Unable to find asset cache")
                .unwrap()
        };

        let item = state
            .cache
            .load_sync(AssetKey::new(&path))
            .map_err(|e| format!("Error while loading asset at `{}`: {}",path.display(), e))
            .unwrap();

        Ok(item)
    }
}

/// Serialize assets without making duplicates
pub struct AssetSeState {
    // cache: AssetCacheAny,
}
