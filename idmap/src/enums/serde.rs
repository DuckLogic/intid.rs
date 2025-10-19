//! Enables serde serialization support for `IdMap`
use core::marker::PhantomData;

use super::EnumMap;
use core::fmt::{self, Formatter};
use intid::EnumId;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};

struct EnumMapVisitor<K: EnumId, V>(PhantomData<EnumMap<K, V>>);

impl<'de, K, V> Visitor<'de> for EnumMapVisitor<K, V>
where
    K: EnumId + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = EnumMap<K, V>;
    #[inline]
    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("an EnumMap")
    }
    #[inline]
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut result = EnumMap::new();
        while let Some((key, value)) = access.next_entry()? {
            result.insert(key, value);
        }
        Ok(result)
    }
}
impl<'de, K, V> Deserialize<'de> for EnumMap<K, V>
where
    K: Deserialize<'de>,
    K: EnumId,
    V: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(EnumMapVisitor(PhantomData))
    }
}
impl<K, V> Serialize for EnumMap<K, V>
where
    K: EnumId,
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
