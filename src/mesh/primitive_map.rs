
#[derive(Debug, Clone)]
pub struct VertexMap<V>
{
    values: Vec<V>,
    free: Vec<u32>
}

impl<V> VertexMap<V>
    where V: Clone
{
    pub fn with_capacity(capacity: usize) -> Self {
        VertexMap { values: Vec::with_capacity(capacity), free: Vec::new() }
    }

    pub fn insert_new(&mut self, value: V) -> Option<VertexID>  {
        if let Some(i) = self.free.pop() {
            let id = VertexID::new(i);
            self.values[i as usize] = value;
            Some(id)
        }
        else {
            self.values.push(value);
            Some(VertexID::new(self.values.len() as u32 - 1))
        }
    }

    pub fn remove(&mut self, id: &VertexID) {
        self.free.push(id.get());
    }

    pub fn len(&self) -> usize {
        self.values.len() - self.free.len()
    }

    pub fn get(&self, id: &VertexID) -> Option<&V> {
        self.values.get(id.get() as usize)
    }

    pub fn get_mut(&mut self, id: &VertexID) -> Option<&mut V> {
        self.values.get_mut(id.get() as usize)
    }

    pub fn iter(&self) -> Box<Iterator<Item = VertexID>> {
        let t: Vec<VertexID> = (0..self.values.len())
            .filter(|i| !self.free.contains(&(*i as u32)))
            .map(|i| VertexID::new(i as u32)).collect();
        Box::new(t.into_iter())
    }
}

use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use crate::mesh::ids::VertexID;

#[derive(Debug, Clone)]
pub struct PrimitiveMap<K, V>
    where K: Eq + std::hash::Hash
{
    map: HashMap<K, V, BuildHasherDefault<PrimitiveIdHasher>>
}

impl<K, V> PrimitiveMap<K, V>
    where K: Eq + std::hash::Hash
{
    pub fn new() -> Self {
        PrimitiveMap { map: HashMap::with_hasher(BuildHasherDefault::<PrimitiveIdHasher>::default()) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        PrimitiveMap {map: HashMap::with_capacity_and_hasher(capacity, BuildHasherDefault::<PrimitiveIdHasher>::default()) }
    }

    pub fn insert(&mut self, id: K, value: V) {
        self.map.insert(id, value);
    }

    pub fn remove(&mut self, id: &K) -> Option<V> {
        self.map.remove(id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get(&self, id: &K) -> Option<&V> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: &K) -> Option<&mut V> {
        self.map.get_mut(id)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<K, V> {
        self.map.iter()
    }

    pub fn contains_key(&self, id: &K) -> bool {
        self.map.contains_key(id)
    }
}

#[derive(Clone)]
struct PrimitiveIdHasher {
    value: u64
}

impl std::default::Default for PrimitiveIdHasher {
    fn default() -> Self {
        PrimitiveIdHasher { value: 0 }
    }
}

impl std::hash::Hasher for PrimitiveIdHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!();
    }

    fn write_u32(&mut self, i: u32) {
        self.value = i as u64;
    }
}