// Implementation is based on
// - https://github.com/hyperium/http/blob/master/src/extensions.rs
// - https://github.com/kardeiz/type-map/blob/master/src/lib.rs

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};

/// Store and retrieve values by `TypeId`.
///
/// Map type that allows storing any `Sync + Send + 'static` type. Instances can
/// be retrieved from [`Request::local`](struct.Request.html#method.local) +
/// [`Response::local`](struct.Response.html#method.local) and
/// [`Request::local_mut`](struct.Request.html#method.local_mut) +
/// [`Response::local_mut`](struct.Response.html#method.local_mut).
#[derive(Default)]
pub struct TypeMap {
    map: Option<HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>>,
}

impl TypeMap {
    /// Create an empty `TypeMap`.
    #[inline]
    pub(crate) fn new() -> Self {
        Self { map: None }
    }

    /// Insert a value into this `TypeMap`.
    ///
    /// If a value of this type already exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .get_or_insert_with(Default::default)
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| (boxed as Box<dyn Any>).downcast().ok().map(|boxed| *boxed))
    }

    /// Check if container contains value for type
    pub fn contains<T: 'static>(&self) -> bool {
        self.map
            .as_ref()
            .and_then(|m| m.get(&TypeId::of::<T>()))
            .is_some()
    }

    /// Get a reference to a value previously inserted on this `TypeMap`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .as_ref()
            .and_then(|m| m.get(&TypeId::of::<T>()))
            .and_then(|boxed| (&**boxed as &(dyn Any)).downcast_ref())
    }

    /// Get a mutable reference to a value previously inserted on this `TypeMap`.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .as_mut()
            .and_then(|m| m.get_mut(&TypeId::of::<T>()))
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any)).downcast_mut())
    }

    /// Remove a value from this `TypeMap`.
    ///
    /// If a value of this type exists, it will be returned.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .as_mut()
            .and_then(|m| m.remove(&TypeId::of::<T>()))
            .and_then(|boxed| (boxed as Box<dyn Any>).downcast().ok().map(|boxed| *boxed))
    }

    /// Clear the `TypeMap` of all inserted values.
    #[inline]
    pub fn clear(&mut self) {
        self.map = None;
    }
}

impl fmt::Debug for TypeMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeMap").finish()
    }
}

// With TypeIds as keys, there's no need to hash them. So we simply use an identy hasher.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_type_map() {
        #[derive(Debug, PartialEq)]
        struct MyType(i32);

        let mut map = TypeMap::new();

        map.insert(5i32);
        map.insert(MyType(10));

        assert_eq!(map.get(), Some(&5i32));
        assert_eq!(map.get_mut(), Some(&mut 5i32));

        assert_eq!(map.remove::<i32>(), Some(5i32));
        assert!(map.get::<i32>().is_none());

        assert_eq!(map.get::<bool>(), None);
        assert_eq!(map.get(), Some(&MyType(10)));
    }
}
