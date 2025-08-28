//! Enables serde serialization support for `IdMap`
use core::marker::PhantomData;

use super::{DirectIdMap, DirectIdSet};
use core::fmt::{self, Formatter};
use intid::IntegerId;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

struct DirectIdMapVisitor<K: IntegerId, V>(PhantomData<DirectIdMap<K, V>>);

impl<'de, K, V> Visitor<'de> for DirectIdMapVisitor<K, V>
where
    K: IntegerId + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = DirectIdMap<K, V>;
    #[inline]
    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("a DirectIdMap")
    }
    #[inline]
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut result = DirectIdMap::new();
        while let Some((key, value)) = access.next_entry()? {
            result.insert(key, value);
        }
        Ok(result)
    }
}
impl<'de, K, V> Deserialize<'de> for DirectIdMap<K, V>
where
    K: Deserialize<'de>,
    K: IntegerId,
    V: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(DirectIdMapVisitor(PhantomData))
    }
}
impl<K, V> Serialize for DirectIdMap<K, V>
where
    K: IntegerId,
    K: Serialize,
    V: Serialize,
{
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(&k, v)?;
        }
        map.end()
    }
}

struct DirectIdSetVisitor<T: IntegerId>(PhantomData<DirectIdSet<T>>);

impl<'de, T> Visitor<'de> for DirectIdSetVisitor<T>
where
    T: IntegerId + Deserialize<'de>,
{
    type Value = DirectIdSet<T>;
    #[inline]
    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("a DirectIdSet")
    }
    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = DirectIdSet::new();
        while let Some(element) = seq.next_element::<T>()? {
            result.insert(element);
        }
        Ok(result)
    }
}
impl<'de, T> Deserialize<'de> for DirectIdSet<T>
where
    T: IntegerId + Deserialize<'de>,
{
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(DirectIdSetVisitor(PhantomData))
    }
}
impl<T> Serialize for DirectIdSet<T>
where
    T: IntegerId + Serialize,
{
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for value in self {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}
