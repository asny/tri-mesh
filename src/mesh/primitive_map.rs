
use crate::mesh::ids::*;

#[derive(Debug, Clone)]
pub(crate) struct IDMap<K, V>
{
    values: Vec<V>,
    indices: Vec<K>,
    free: Vec<K>
}

impl<K: 'static, V> IDMap<K, V>
    where K: ID
{
    pub fn with_capacity(capacity: usize) -> Self {
        IDMap { values: Vec::with_capacity(capacity), indices: Vec::new(), free: Vec::new() }
    }

    pub fn insert_new(&mut self, value: V) -> Option<K>  {
        let id = if let Some(i) = self.free.pop() {
            self.values[i.get() as usize] = value;
            i
        }
        else {
            self.values.push(value);
            K::new(self.values.len() as u32 - 1)
        };
        self.indices.push(id);
        Some(id)
    }

    pub fn remove(&mut self, id: K) {
        self.free.push(id);
        self.indices.retain(|i| *i != id);
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn get(&self, id: K) -> Option<&V> {
        self.values.get(id.get() as usize)
    }

    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.values.get_mut(id.get() as usize)
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = K>> {
        Box::new(self.indices.clone().into_iter())
    }
}