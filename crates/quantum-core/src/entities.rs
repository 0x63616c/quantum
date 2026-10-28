//! Ordered runtime arenas; portable identities remain UUIDs on the wire.
use crate::{Comment, Layer, Uuid};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use slotmap::{Key, SlotMap, new_key_type};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::OnceLock,
};

new_key_type! {
    /// Runtime layer identity, independent of display order.
    pub struct LayerId;
    /// Runtime discussion identity, independent of display order.
    pub struct CommentId;
}

/// Portable identity used to resolve a runtime arena key.
pub trait Entity {
    /// Existing persisted UUID, never a serialized arena key.
    fn uuid(&self) -> Uuid;
}
impl Entity for Layer {
    fn uuid(&self) -> Uuid {
        self.id
    }
}
impl Entity for Comment {
    fn uuid(&self) -> Uuid {
        self.id
    }
}

/// Stable runtime storage with separate presentation order and UUID resolution.
///
/// Keys survive reordering and unrelated deletion. Mutable access invalidates
/// the derived UUID index, including edits to a public portable `id` field.
/// Serialization is the original ordered JSON array, without runtime keys.
#[derive(Clone, Debug)]
pub struct Entities<K: Key, T> {
    entries: SlotMap<K, T>,
    order: Vec<K>,
    ids: OnceLock<HashMap<Uuid, K>>,
}
/// Ordered document layers.
pub type Layers = Entities<LayerId, Layer>;
/// Ordered document discussions.
pub type Comments = Entities<CommentId, Comment>;
impl<K: Key, T> Default for Entities<K, T> {
    fn default() -> Self {
        Self {
            entries: SlotMap::with_key(),
            order: Vec::new(),
            ids: OnceLock::new(),
        }
    }
}
impl<K: Key, T> Entities<K, T> {
    /// Number of retained entities.
    pub fn len(&self) -> usize {
        self.order.len()
    }
    /// Whether no entities remain.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
    /// Entities in presentation order.
    pub fn iter(&self) -> EntityIter<'_, K, T> {
        EntityIter {
            entries: &self.entries,
            order: self.order.iter(),
        }
    }
    /// Mutable entities in presentation order, invalidating portable-ID lookup.
    pub fn iter_mut(&mut self) -> std::vec::IntoIter<&mut T> {
        self.ids.take();
        let ranks: HashMap<_, _> = self
            .order
            .iter()
            .enumerate()
            .map(|(i, k)| (*k, i))
            .collect();
        let mut entries: Vec<_> = self.entries.iter_mut().collect();
        entries.sort_unstable_by_key(|(key, _)| ranks[key]);
        entries
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>()
            .into_iter()
    }
    /// Keys in presentation order.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = K> + ExactSizeIterator + '_ {
        self.order.iter().copied()
    }
    /// Resolve a key, rejecting deleted generations.
    pub fn get(&self, key: K) -> Option<&T> {
        self.entries.get(key)
    }
    /// Mutably resolve a key, rejecting deleted generations.
    pub fn get_mut(&mut self, key: K) -> Option<&mut T> {
        self.ids.take();
        self.entries.get_mut(key)
    }
    /// Append an entity, returning its stable runtime identity.
    pub fn push(&mut self, value: T) -> K {
        self.ids.take();
        let key = self.entries.insert(value);
        self.order.push(key);
        key
    }
    /// Insert at a presentation position, returning its runtime identity.
    pub fn insert(&mut self, index: usize, value: T) -> K {
        assert!(index <= self.len(), "entity insertion index");
        self.ids.take();
        let key = self.entries.insert(value);
        self.order.insert(index, key);
        key
    }
    /// Remove a presentation position and invalidate its key.
    #[expect(clippy::expect_used, reason = "internal invariant: ordered arena key")]
    pub fn remove(&mut self, index: usize) -> T {
        self.ids.take();
        self.entries
            .remove(self.order.remove(index))
            .expect("ordered arena key")
    }
    /// Keep matching entities, preserving their keys and presentation order.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        self.ids.take();
        self.order.retain(|key| {
            if keep(&self.entries[*key]) {
                true
            } else {
                self.entries.remove(*key);
                false
            }
        });
    }
    /// Remove all entities, invalidating all retained keys.
    pub fn clear(&mut self) {
        self.ids.take();
        self.order.clear();
        self.entries.clear();
    }
    /// Reverse presentation order without changing keys.
    pub fn reverse(&mut self) {
        self.order.reverse();
    }
    /// Exchange two presentation positions without changing keys.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.order.swap(a, b);
    }
    /// Sort presentation order without changing keys.
    pub fn sort_by_key<B: Ord>(&mut self, mut key: impl FnMut(&T) -> B) {
        self.order.sort_by_key(|id| key(&self.entries[*id]));
    }
    /// Presentation position of a runtime key.
    pub fn position(&self, key: K) -> Option<usize> {
        self.order.iter().position(|candidate| *candidate == key)
    }
    /// Insert immediately above a retained key in presentation order.
    pub fn insert_after(&mut self, source: K, value: T) -> Option<K> {
        let index = self.position(source)?;
        Some(self.insert(index + 1, value))
    }
    /// Extend presentation order with fresh keys.
    pub fn extend(&mut self, values: impl IntoIterator<Item = T>) {
        for value in values {
            self.push(value);
        }
    }
    /// First entity in presentation order.
    pub fn first(&self) -> Option<&T> {
        self.order.first().map(|key| &self.entries[*key])
    }
}
impl<K: Key, T: Entity> Entities<K, T> {
    /// Resolve an existing portable UUID in constant time after index construction.
    pub fn key(&self, id: Uuid) -> Option<K> {
        self.ids
            .get_or_init(|| {
                self.entries
                    .iter()
                    .map(|(key, value)| (value.uuid(), key))
                    .collect()
            })
            .get(&id)
            .copied()
    }
    /// Reorder by portable IDs after the caller validates the permutation.
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: validated entity permutation"
    )]
    pub(crate) fn reorder(&mut self, order: &[Uuid]) {
        self.order = order
            .iter()
            .map(|id| self.key(*id).expect("validated entity permutation"))
            .collect();
    }
    /// Find by portable UUID at a command or persistence boundary.
    pub fn by_uuid(&self, id: Uuid) -> Option<&T> {
        self.key(id).and_then(|key| self.get(key))
    }
    /// Mutably find by portable UUID at a command boundary.
    pub fn by_uuid_mut(&mut self, id: Uuid) -> Option<&mut T> {
        let key = self.key(id)?;
        self.get_mut(key)
    }
}
impl<K: Key, T: PartialEq> PartialEq for Entities<K, T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl<K: Key, T: Eq> Eq for Entities<K, T> {}
impl<K: Key, T> From<Vec<T>> for Entities<K, T> {
    fn from(values: Vec<T>) -> Self {
        let mut result = Self::default();
        result.extend(values);
        result
    }
}
impl<K: Key, T> FromIterator<T> for Entities<K, T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut result = Self::default();
        result.extend(values);
        result
    }
}
impl<K: Key, T> Index<usize> for Entities<K, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.entries[self.order[index]]
    }
}
impl<K: Key, T> IndexMut<usize> for Entities<K, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.ids.take();
        &mut self.entries[self.order[index]]
    }
}
impl<K: Key, T: Serialize> Serialize for Entities<K, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
impl<'de, K: Key, T: Deserialize<'de>> Deserialize<'de> for Entities<K, T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<T>::deserialize(deserializer).map(Into::into)
    }
}
/// Borrowed entities in presentation order, without allocating.
pub struct EntityIter<'a, K: Key, T> {
    entries: &'a SlotMap<K, T>,
    order: std::slice::Iter<'a, K>,
}
impl<'a, K: Key, T> Iterator for EntityIter<'a, K, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.order.next().map(|key| &self.entries[*key])
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.order.size_hint()
    }
}
impl<K: Key, T> DoubleEndedIterator for EntityIter<'_, K, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.order.next_back().map(|key| &self.entries[*key])
    }
}
impl<K: Key, T> ExactSizeIterator for EntityIter<'_, K, T> {}
impl<'a, K: Key, T> IntoIterator for &'a Entities<K, T> {
    type Item = &'a T;
    type IntoIter = EntityIter<'a, K, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K: Key, T> IntoIterator for &'a mut Entities<K, T> {
    type Item = &'a mut T;
    type IntoIter = std::vec::IntoIter<&'a mut T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<K: Key, T> IntoIterator for Entities<K, T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    #[expect(clippy::expect_used, reason = "internal invariant: ordered arena key")]
    fn into_iter(mut self) -> Self::IntoIter {
        self.order
            .into_iter()
            .map(|key| self.entries.remove(key).expect("ordered arena key"))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_survive_reorder_and_reject_deleted_generations() {
        let mut layers = Layers::default();
        let a = layers.push(Layer::new(Uuid::from_u128(1), "A"));
        let b = layers.push(Layer::new(Uuid::from_u128(2), "B"));
        layers.swap(0, 1);
        assert_eq!(layers.keys().collect::<Vec<_>>(), vec![b, a]);
        assert_eq!(layers.key(Uuid::from_u128(1)), Some(a));
        layers.remove(1);
        let c = layers.push(Layer::new(Uuid::from_u128(1), "C"));
        assert_ne!(a, c);
        assert!(layers.get(a).is_none());
        assert_eq!(layers.get(b).unwrap().name, "B");
        assert_eq!(layers.key(Uuid::from_u128(1)), Some(c));
        layers.clear();
        assert!(layers.get(b).is_none());
        assert!(layers.get(c).is_none());
    }
    #[test]
    fn mutable_ids_invalidate_lookup_and_iteration_preserves_order() {
        let mut layers: Layers = vec![
            Layer::new(Uuid::from_u128(1), "A"),
            Layer::new(Uuid::from_u128(2), "B"),
        ]
        .into();
        let a = layers.key(Uuid::from_u128(1)).unwrap();
        layers.swap(0, 1);
        for (i, layer) in layers.iter_mut().enumerate() {
            layer.name = i.to_string();
        }
        assert_eq!(layers.get(a).unwrap().name, "1");
        layers.get_mut(a).unwrap().id = Uuid::from_u128(3);
        assert!(layers.key(Uuid::from_u128(1)).is_none());
        assert_eq!(layers.key(Uuid::from_u128(3)), Some(a));
        layers.by_uuid_mut(Uuid::from_u128(3)).unwrap().id = Uuid::from_u128(4);
        assert_eq!(layers.key(Uuid::from_u128(4)), Some(a));
        layers[1].id = Uuid::from_u128(5);
        assert_eq!(layers.key(Uuid::from_u128(5)), Some(a));
        layers.retain(|l| l.id != Uuid::from_u128(5));
        assert!(layers.key(Uuid::from_u128(5)).is_none());
        assert!(layers.get(a).is_none());
    }
    #[test]
    fn serialization_and_equality_ignore_runtime_allocation_history() {
        let mut layers = Layers::default();
        layers.push(Layer::new(Uuid::from_u128(1), "Discarded"));
        layers.clear();
        layers.push(Layer::new(Uuid::from_u128(2), "Retained"));
        let bytes = serde_json::to_vec(&layers).unwrap();
        let restored: Layers = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(layers, restored);
        assert_ne!(layers.keys().next(), restored.keys().next());
        let values: Vec<Layer> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&values).unwrap(), bytes);
    }
}
