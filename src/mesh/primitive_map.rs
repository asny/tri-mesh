
use crate::mesh::ids::*;

#[derive(Debug, Clone)]
pub struct VertexMap<V>
{
    values: Vec<V>,
    free: Vec<u32>
}

impl<V> VertexMap<V>
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

    pub fn remove(&mut self, id: VertexID) {
        self.free.push(id.get());
    }

    pub fn len(&self) -> usize {
        self.values.len() - self.free.len()
    }

    pub fn get(&self, id: VertexID) -> Option<&V> {
        self.values.get(id.get() as usize)
    }

    pub fn get_mut(&mut self, id: VertexID) -> Option<&mut V> {
        self.values.get_mut(id.get() as usize)
    }

    pub fn iter(&self) -> Box<Iterator<Item = VertexID>> {
        let t: Vec<VertexID> = (0..self.values.len())
            .filter(|i| !self.free.contains(&(*i as u32)))
            .map(|i| VertexID::new(i as u32)).collect();
        Box::new(t.into_iter())
    }
}


#[derive(Debug, Clone)]
pub struct HalfEdgeMap<V>
{
    values: Vec<V>,
    free: Vec<u32>
}

impl<V> HalfEdgeMap<V>
{
    pub fn with_capacity(capacity: usize) -> Self {
        HalfEdgeMap { values: Vec::with_capacity(capacity), free: Vec::new() }
    }

    pub fn insert_new(&mut self, value: V) -> Option<HalfEdgeID>  {
        if let Some(i) = self.free.pop() {
            let id = HalfEdgeID::new(i);
            self.values[i as usize] = value;
            Some(id)
        }
        else {
            self.values.push(value);
            Some(HalfEdgeID::new(self.values.len() as u32 - 1))
        }
    }

    pub fn remove(&mut self, id: HalfEdgeID) {
        self.free.push(id.get());
    }

    pub fn len(&self) -> usize {
        self.values.len() - self.free.len()
    }

    pub fn get(&self, id: HalfEdgeID) -> Option<&V> {
        self.values.get(id.get() as usize)
    }

    pub fn get_mut(&mut self, id: HalfEdgeID) -> Option<&mut V> {
        self.values.get_mut(id.get() as usize)
    }

    pub fn iter(&self) -> Box<Iterator<Item = HalfEdgeID>> {
        let t: Vec<HalfEdgeID> = (0..self.values.len())
            .filter(|i| !self.free.contains(&(*i as u32)))
            .map(|i| HalfEdgeID::new(i as u32)).collect();
        Box::new(t.into_iter())
    }
}

#[derive(Debug, Clone)]
pub struct FaceMap<V>
{
    values: Vec<V>,
    free: Vec<u32>
}

impl<V> FaceMap<V>
{
    pub fn with_capacity(capacity: usize) -> Self {
        FaceMap { values: Vec::with_capacity(capacity), free: Vec::new() }
    }

    pub fn insert_new(&mut self, value: V) -> Option<FaceID>  {
        if let Some(i) = self.free.pop() {
            let id = FaceID::new(i);
            self.values[i as usize] = value;
            Some(id)
        }
        else {
            self.values.push(value);
            Some(FaceID::new(self.values.len() as u32 - 1))
        }
    }

    pub fn remove(&mut self, id: FaceID) {
        self.free.push(id.get());
    }

    pub fn len(&self) -> usize {
        self.values.len() - self.free.len()
    }

    pub fn get(&self, id: FaceID) -> Option<&V> {
        self.values.get(id.get() as usize)
    }

    pub fn get_mut(&mut self, id: FaceID) -> Option<&mut V> {
        self.values.get_mut(id.get() as usize)
    }

    pub fn iter(&self) -> Box<Iterator<Item = FaceID>> {
        let t: Vec<FaceID> = (0..self.values.len())
            .filter(|i| !self.free.contains(&(*i as u32)))
            .map(|i| FaceID::new(i as u32)).collect();
        Box::new(t.into_iter())
    }
}