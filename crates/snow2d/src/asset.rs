/*!

Reference-counted asset pointer and asset cache

TODO: async loading and dynamic loading

*/

#![allow(dead_code)]

pub use std::io::Result;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Get asset path relative to `assets` directory
/// TODO: remove `unsafe`
pub unsafe fn path(path: impl AsRef<Path>) -> PathBuf {
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
            item: self.item.as_ref().map(|x| Arc::clone(&x)),
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
    pub fn get<'a>(&'a self) -> Option<impl Deref<Target = T> + 'a> {
        self.item.as_ref()?.lock().ok()
    }

    /// Tries to `&mut T`, fails if the asset is not loaded or panics ([`Mutex`] under the hood)
    ///
    /// This step is for asynchrounous loading and hot reloaidng.
    pub fn get_mut<'a>(&'a mut self) -> Option<impl DerefMut + Deref<Target = T> + 'a> {
        self.item.as_mut()?.lock().ok()
    }
}

pub type AssetKey = Path;

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
    assets: Vec<AssetCacheEntry<T>>,
    loader: T::Loader,
}

#[derive(Debug)]
pub struct AssetCacheEntry<T: AssetItem> {
    id: AssetId,
    path: PathBuf,
    item: Asset<T>,
}

impl<T: AssetItem> AssetCacheT<T> {
    pub fn new(loader: T::Loader) -> Self {
        Self {
            assets: Vec::with_capacity(16),
            loader,
        }
    }

    pub fn load_sync(&mut self, key: impl AsRef<AssetKey>) -> Result<Asset<T>> {
        let id = AssetId::from_key(key.as_ref());
        if let Some(a) = self.find_cache(&id) {
            Ok(a)
        } else {
            self.load_new_sync(id)
        }
    }

    fn find_cache(&mut self, id: &AssetId) -> Option<Asset<T>> {
        self.assets
            .iter()
            .find(|a| a.id == *id)
            .map(|a| a.item.clone())
    }

    fn load_new_sync(&mut self, id: AssetId) -> Result<Asset<T>> {
        let asset = Asset {
            item: {
                let path = unsafe { self::path(&id.identity) };
                let item = self.loader.load(&path)?;
                Some(Arc::new(Mutex::new(item)))
            },
        };

        let path = unsafe { self::path(&id.identity) };

        let entry = AssetCacheEntry {
            id,
            path,
            item: asset.clone(),
        };
        self.assets.push(entry);

        Ok(asset)
    }
}

#[derive(Debug)]
pub struct AssetCacheAny {
    caches: HashMap<TypeId, Box<dyn Any>>,
}

impl AssetCacheAny {
    pub fn new() -> Self {
        Self {
            caches: HashMap::with_capacity(16),
        }
    }

    pub fn add_cache<T: AssetItem>(&mut self, cache: AssetCacheT<T>) {
        self.caches.insert(TypeId::of::<T>(), Box::new(cache));
    }

    pub fn cache_mut<T: AssetItem>(&mut self) -> Option<&mut AssetCacheT<T>> {
        let boxed = self.caches.get_mut(&TypeId::of::<T>()).unwrap();
        boxed.downcast_mut::<AssetCacheT<T>>()
    }

    pub fn load_sync<T: AssetItem>(&mut self, key: impl AsRef<AssetKey>) -> Result<Asset<T>> {
        // TODO: don't unwrap
        self.cache_mut::<T>().unwrap().load_sync(key)
    }
}
