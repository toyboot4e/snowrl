/*!
Utilities
*/

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

use derivative::Derivative;
use downcast_rs::Downcast;
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use snow2d::asset::AssetKey;

// /// Storage of any type
// #[derive(Debug)]
// pub struct AnyMap {
//     map: HashMap<TypeId, Box<dyn Any>>,
// }
//
// impl AnyMap {
//     pub fn new() -> Self {
//         Self {
//             map: Default::default(),
//         }
//     }
//
//     pub fn with_capacity(cap: usize) -> Self {
//         Self {
//             map: HashMap::with_capacity(cap),
//         }
//     }
//
//     pub fn get<T: 'static>(&self) -> Option<&T> {
//         self.map
//             .get(&TypeId::of::<T>())
//             .map(|any| any.downcast_ref::<T>().unwrap())
//     }
// }

/// Marker for data that define "type" of instances ([type objects])
///
/// [type objects]: https://gameprogrammingpatterns.com/type-object.html
pub trait TypeObject: std::fmt::Debug + Sized {
    fn from_type_key(key: &TypeObjectId<Self>) -> anyhow::Result<Rc<Self>>
    where
        Self: 'static,
    {
        TypeObjectStorage::get_type_object(key)
            .ok_or_else(|| anyhow::anyhow!(format!("Unable to find type object for {:?}", key)))
    }
}

/// Id for fly objects are shared among concrete types
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeObjectId<T: TypeObject> {
    key: String,
    _marker: PhantomData<fn() -> T>,
}

impl<'a, T: TypeObject> From<&'a str> for TypeObjectId<T> {
    fn from(s: &'a str) -> Self {
        TypeObjectId {
            key: s.to_string(),
            _marker: PhantomData,
        }
    }
}

impl<'de, T: TypeObject> serde::de::Deserialize<'de> for TypeObjectId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let key = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        Ok(Self {
            key,
            _marker: PhantomData,
        })
    }
}

impl<T: TypeObject> serde::ser::Serialize for TypeObjectId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.key.serialize(serializer)
    }
}

impl<T: TypeObject> TypeObjectId<T> {
    pub fn from_raw(s: String) -> Self {
        Self {
            key: s,
            _marker: PhantomData,
        }
    }

    pub fn raw(&self) -> &str {
        &self.key
    }
}

/// Static storage of type objects
#[derive(Debug)]
pub struct TypeObjectStorage {
    maps: HashMap<TypeId, Box<dyn Any>>,
}

static mut STORAGE: OnceCell<TypeObjectStorage> = OnceCell::new();

impl TypeObjectStorage {
    pub fn init() -> Result<(), Self> {
        unsafe {
            STORAGE.set(TypeObjectStorage {
                maps: HashMap::with_capacity(16),
            })
        }
    }

    // TODO: maybe use TypeObjectStorageBuilder
    pub fn register_type_objects<
        'a,
        T: TypeObject + 'static + DeserializeOwned,
        U: Into<AssetKey<'a>>,
    >(
        key: U,
    ) -> anyhow::Result<()> {
        unsafe {
            let s = STORAGE
                .get_mut()
                .expect("TypeObjectStorage is not initialized");

            let map: HashMap<TypeObjectId<T>, T> = snow2d::asset::deserialize_ron(key)?;
            let map: HashMap<TypeObjectId<T>, Rc<T>> = map
                .into_iter()
                .map(|(key, value)| (key, Rc::new(value)))
                .collect::<HashMap<_, _>>();

            anyhow::ensure!(
                s.maps
                    .insert(TypeId::of::<T>(), Box::new(TypeObjectMap { data: map }),)
                    .is_none(),
                "Registring type objects twice for type `{}`",
                std::any::type_name::<T>(),
            );

            Ok(())
        }
    }

    fn get_any<T: TypeObject + 'static>() -> &'static Box<dyn Any> {
        unsafe {
            let s = STORAGE.get().expect("TypeObjectStorage is not initialized");

            s.maps.get(&TypeId::of::<T>()).unwrap_or_else(|| {
                panic!(
                    "No TypeObjectMap found for type `{}`",
                    std::any::type_name::<T>()
                )
            })
        }
    }

    pub fn get_map<T: TypeObject>() -> Option<&'static TypeObjectMap<T>> {
        Self::get_any::<T>().downcast_ref::<TypeObjectMap<T>>()
    }

    pub fn get_type_object<T: TypeObject + 'static>(id: &TypeObjectId<T>) -> Option<Rc<T>> {
        let map = Self::get_map::<T>()?;
        map.get(id)
    }
}

pub struct TypeObjectMap<T: TypeObject> {
    // TODO: maybe use Pool
    data: HashMap<TypeObjectId<T>, Rc<T>>,
}

impl<T: TypeObject> TypeObjectMap<T> {
    pub fn get(&self, id: &TypeObjectId<T>) -> Option<Rc<T>> {
        self.data.get(id).map(|rc| Rc::clone(rc))
    }
}

/// `Embedded` | `Reference` representation of a type object for serde
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeObjectSerdeRepr<T: TypeObject> {
    Embedded(T),
    Reference(TypeObjectId<T>),
}

pub trait SerdeFromTypeObject {
    type TypeObject: TypeObject;
    fn from_type_object(obj: Self::TypeObject) -> Self;
}

// macro_rules! impl_from_type_object {
//     ($T:ty, $U:ty) => {
//         impl<'de> serde::de::Deserialize<'de> for U
//             fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//             where
//                 D: serde::de::Deserializer<'de>,
//         {
//             fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
//                 let repr = TypeObjectSerdeRepr::deserialize(deserializer)?;
//                 let type_object = match repr {
//                     TypeObjectSerdeRepr::Embedded(type_object) => type_object,
//                     TypeObjectSerdeRepr::Reference(id) => TypeObjectStorage::get_type_object(id),
//                 };
//                 Ok(<U as SerdeFromTypeObject>::from_type_object(type_object))
//             }
//         }
//     }
// }

// /// Upcast objects to trait marked as [`Downcast`]
// pub struct TraitStorage<T: DownCast> {
//     data: Vec<Box<dyn Any>>,
// }
//
// impl TraitStorage {
//     pub fn get<T>(&self) -> Option<&T> {
//         self.data.iter().filter_map(|data|data.downcast_ref::<T>())
//     }
// }

/// Raw double buffer with interpolation value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoubleTrack<T> {
    /// Front
    pub a: T,
    /// Back
    pub b: T,
    /// Interpolation value
    pub t: f32,
}

impl<T: Default> Default for DoubleTrack<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            t: Default::default(),
        }
    }
}

impl<T> DoubleTrack<T> {
    /// TODO: maybe improve efficiency
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

/// Double buffer that can internally swap buffers without copy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleSwap<T> {
    /// Front buffer at initial state
    a: T,
    /// Back buffer at initial state
    b: T,
    /// True then `a` is front
    counter: bool,
}

impl<T: Default> Default for DoubleSwap<T> {
    fn default() -> Self {
        Self {
            a: T::default(),
            b: T::default(),
            counter: true,
        }
    }
}

impl<T> DoubleSwap<T> {
    pub fn new(a: T, b: T) -> Self {
        Self {
            a,
            b,
            counter: true,
        }
    }

    /// Swaps front/back buffers
    pub fn swap(&mut self) {
        self.counter = !self.counter;
    }

    pub fn unwrap(self) -> [T; 2] {
        if self.counter {
            [self.a, self.b]
        } else {
            [self.b, self.a]
        }
    }

    pub fn into_a(self) -> T {
        if self.counter {
            self.a
        } else {
            self.b
        }
    }

    pub fn into_b(self) -> T {
        if self.counter {
            self.b
        } else {
            self.a
        }
    }

    /// Front
    pub fn a(&self) -> &T {
        if self.counter {
            &self.a
        } else {
            &self.b
        }
    }

    pub fn a_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.a
        } else {
            &mut self.b
        }
    }

    pub fn set_a(&mut self, x: T) {
        if self.counter {
            self.a = x;
        } else {
            self.b = x;
        }
    }

    /// Back
    pub fn b(&self) -> &T {
        if self.counter {
            &self.b
        } else {
            &self.a
        }
    }

    pub fn b_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.b
        } else {
            &mut self.a
        }
    }

    pub fn set_b(&mut self, x: T) {
        if self.counter {
            self.b = x;
        } else {
            self.a = x;
        }
    }
}
