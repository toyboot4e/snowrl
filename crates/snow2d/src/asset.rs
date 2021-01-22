/*!

Asset cache and reference-counted asset references

TODO fixes:

* release cache on no owner

TODO features:

* weak pointer?
* serde (interning `Arc` asset handles)
* async loading
* dynamic loading

*/

#![allow(dead_code)]

pub use std::io::Result;

use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashMap,
    fmt, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use downcast_rs::{impl_downcast, Downcast};

/// Get asset path relative to `assets` directory
/// TODO: remove `unsafe`
pub fn path(path: impl AsRef<Path>) -> PathBuf {
    // TODO: supply appropreate root path
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets = PathBuf::from(root).join("assets");
    assets.join(path)
}

/// Asset data
pub trait AssetItem: fmt::Debug + Sized + 'static {
    type Loader: AssetLoader<Item = Self>;
}

pub trait AssetLoader: fmt::Debug + Sized + 'static {
    type Item: AssetItem;
    fn load(&mut self, path: &Path) -> Result<Self::Item>;
}

/// Shared ownership of an [`AssetItem`]
#[derive(Debug)]
pub struct Asset<T: AssetItem> {
    item: Option<Arc<Mutex<T>>>,
}

impl<T: AssetItem> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.as_ref().map(|x| Arc::clone(x)),
        }
    }
}

impl<T: AssetItem> Asset<T> {
    pub fn empty() -> Self {
        Self { item: None }
    }

    pub fn is_loaded() -> bool {
        true
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

/// TODO: use newtype struct while enabling static construction (or use IntoAssetKey)
pub struct AssetKey<'a>(Cow<'a, Path>);

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

/// Cache of a specific [`AssetItem`] type
#[derive(Debug)]
pub struct AssetCacheT<T: AssetItem> {
    entries: Vec<AssetCacheEntry<T>>,
    loader: T::Loader,
}

#[derive(Debug)]
struct AssetCacheEntry<T: AssetItem> {
    id: AssetId,
    path: PathBuf,
    asset: Asset<T>,
}

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
            log::trace!("(cache found for {})", key.display());
            Ok(a)
        } else {
            log::debug!("loading {}", key.display());
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
        let path = self::path(&id.identity);

        let asset = Asset {
            item: {
                let item = self.loader.load(&path)?;
                Some(Arc::new(Mutex::new(item)))
            },
        };

        let entry = AssetCacheEntry {
            id,
            path,
            asset: asset.clone(),
        };
        self.entries.push(entry);

        Ok(asset)
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
