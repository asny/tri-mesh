use std::cell::{RefCell};
use crate::mesh::ids::*;
use crate::mesh::math::Vec3;

#[derive(Clone, Debug)]
pub(crate) struct ConnectivityInfo {
    vertices: RefCell<IDMap<VertexID, Vertex>>,
    halfedges: RefCell<IDMap<HalfEdgeID, HalfEdge>>,
    faces: RefCell<IDMap<FaceID, Face>>
}

impl ConnectivityInfo {
    pub fn new(no_vertices: usize, no_faces: usize) -> ConnectivityInfo
    {
        ConnectivityInfo {
            vertices: RefCell::new(IDMap::with_capacity(no_vertices)),
            halfedges: RefCell::new(IDMap::with_capacity(4 * no_faces)),
            faces: RefCell::new(IDMap::with_capacity(no_faces))
        }
    }

    pub fn no_vertices(&self) -> usize
    {
        RefCell::borrow(&self.vertices).len()
    }

    pub fn no_halfedges(&self) -> usize
    {
        RefCell::borrow(&self.halfedges).len()
    }

    pub fn no_faces(&self) -> usize
    {
        RefCell::borrow(&self.faces).len()
    }

    // Creates a face and the three internal half-edges and connects them to eachother and to the three given vertices
    pub fn create_face(&self, vertex_id1: VertexID, vertex_id2: VertexID, vertex_id3: VertexID) -> FaceID
    {
        let id = self.new_face();

        // Create inner half-edges
        let halfedge1 = self.new_halfedge(Some(vertex_id2), None, Some(id));
        let halfedge3 = self.new_halfedge(Some(vertex_id1), Some(halfedge1), Some(id));
        let halfedge2 = self.new_halfedge(Some(vertex_id3), Some(halfedge3), Some(id));

        self.set_halfedge_next(halfedge1, Some(halfedge2));

        self.set_vertex_halfedge(vertex_id1, Some(halfedge1));
        self.set_vertex_halfedge(vertex_id2, Some(halfedge2));
        self.set_vertex_halfedge(vertex_id3, Some(halfedge3));

        self.set_face_halfedge(id, halfedge1);

        id
    }

    pub fn create_face_with_existing_halfedge(&self, vertex_id1: VertexID, vertex_id2: VertexID, vertex_id3: VertexID, halfedge_id: HalfEdgeID) -> FaceID
    {
        let id = self.new_face();

        // Create inner half-edges
        let halfedge3 = self.new_halfedge(Some(vertex_id1), Some(halfedge_id), Some(id));
        let halfedge2 = self.new_halfedge(Some(vertex_id3), Some(halfedge3), Some(id));

        self.set_halfedge_next(halfedge_id, Some(halfedge2));
        self.set_halfedge_face(halfedge_id, Some(id));

        self.set_vertex_halfedge(vertex_id1, Some(halfedge_id));
        self.set_vertex_halfedge(vertex_id2, Some(halfedge2));
        self.set_vertex_halfedge(vertex_id3, Some(halfedge3));

        self.set_face_halfedge(id, halfedge_id);

        id
    }

    pub fn new_vertex(&self, position: Vec3) -> VertexID
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);
        vertices.insert_new(Vertex { halfedge: None, position }).unwrap()
    }

    pub fn new_halfedge(&self, vertex: Option<VertexID>, next: Option<HalfEdgeID>, face: Option<FaceID>) -> HalfEdgeID
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        halfedges.insert_new(HalfEdge { vertex, twin: None, next, face }).unwrap()
    }

    fn new_face(&self) -> FaceID
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);
        faces.insert_new(Face { halfedge: None }).unwrap()
    }

    pub fn remove_vertex(&self, vertex_id: VertexID)
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);
        vertices.remove(vertex_id);
    }

    pub fn remove_halfedge(&self, halfedge_id: HalfEdgeID)
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        let halfedge = halfedges.get(halfedge_id).unwrap();
        if let Some(twin_id) = halfedge.twin
        {
            halfedges.get_mut(twin_id).unwrap().twin = None;
        }
        halfedges.remove(halfedge_id);
    }

    pub fn remove_face(&self, face_id: FaceID)
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);
        faces.remove(face_id);
    }

    pub fn set_vertex_halfedge(&self, id: VertexID, val: Option<HalfEdgeID>)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = val;
    }

    pub fn set_halfedge_next(&self, id: HalfEdgeID, val: Option<HalfEdgeID>)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().next = val;
    }

    pub fn set_halfedge_twin(&self, id1: HalfEdgeID, id2: HalfEdgeID)
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        halfedges.get_mut(id1).unwrap().twin = Some(id2);
        halfedges.get_mut(id2).unwrap().twin = Some(id1);
    }

    pub fn set_halfedge_vertex(&self, id: HalfEdgeID, val: VertexID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().vertex = Some(val);
    }

    pub fn set_halfedge_face(&self, id: HalfEdgeID, val: Option<FaceID>)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().face = val;
    }

    pub fn set_face_halfedge(&self, id: FaceID, val: HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces).get_mut(id).unwrap().halfedge = Some(val);
    }

    pub fn vertex_iterator(&self) -> Box<dyn Iterator<Item = VertexID>>
    {
        RefCell::borrow(&self.vertices).iter()
    }

    pub fn halfedge_iterator(&self) -> Box<dyn Iterator<Item=HalfEdgeID>>
    {
        RefCell::borrow(&self.halfedges).iter()
    }

    pub fn face_iterator(&self) -> Box<dyn Iterator<Item = FaceID>>
    {
        RefCell::borrow(&self.faces).iter()
    }

    pub fn vertex_halfedge(&self, vertex_id: VertexID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge(&self, halfedge_id: HalfEdgeID) -> Option<HalfEdge>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).and_then(|halfedge| Some(halfedge.clone()))
    }

    pub fn face_halfedge(&self, face_id: FaceID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.faces).get(face_id).unwrap().halfedge.clone()
    }

    pub fn position(&self, vertex_id: VertexID) -> Vec3
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().position
    }

    pub fn set_position(&self, vertex_id: VertexID, position: Vec3)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(vertex_id).unwrap().position = position;
    }
}

impl std::fmt::Display for ConnectivityInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** VERTICES: ****")?;
        let vertices = RefCell::borrow(&self.vertices);
        writeln!(f, "Count: {}", vertices.len())?;
        for id in vertices.iter() {
            writeln!(f, "{}: {:?}", id, vertices.get(id))?;
        }
        writeln!(f, "**** Halfedges: ****")?;
        let halfedges = RefCell::borrow(&self.halfedges);
        writeln!(f, "Count: {}", halfedges.len())?;
        for id in halfedges.iter() {
            writeln!(f, "{}: {:?}", id, halfedges.get(id))?;
        }
        writeln!(f, "**** Faces: ****")?;
        let faces = RefCell::borrow(&self.faces);
        writeln!(f, "Count: {}", faces.len())?;
        for id in faces.iter() {
            writeln!(f, "{}: {:?}", id, faces.get(id))?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub halfedge: Option<HalfEdgeID>,
    pub position: Vec3
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct HalfEdge {
    pub vertex: Option<VertexID>,
    pub twin: Option<HalfEdgeID>,
    pub next: Option<HalfEdgeID>,
    pub face: Option<FaceID>
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Face {
    pub halfedge: Option<HalfEdgeID>
}

#[derive(Debug, Clone)]
pub(crate) struct IDMap<K, V>
{
    values: Vec<V>,
    free: Vec<K>
}

use std::collections::HashSet;

impl<K: 'static, V> IDMap<K, V>
    where K: ID
{
    pub fn with_capacity(capacity: usize) -> Self {
        IDMap { values: Vec::with_capacity(capacity), free: Vec::new() }
    }

    pub fn insert_new(&mut self, value: V) -> Option<K>  {
        let id = if let Some(i) = self.free.pop() {
            self.values[i.deref() as usize] = value;
            i
        }
        else {
            self.values.push(value);
            K::new(self.values.len() as u32 - 1)
        };
        Some(id)
    }

    pub fn remove(&mut self, id: K) {
        self.free.push(id);
    }

    pub fn len(&self) -> usize {
        self.values.len() - self.free.len()
    }

    pub fn get(&self, id: K) -> Option<&V> {
        self.values.get(id.deref() as usize)
    }

    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.values.get_mut(id.deref() as usize)
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = K>> {
		let free: HashSet<_> = self.free.iter().cloned().collect();
		Box::new(   (0 .. self.values.len() as u32)
					.map(|i| K::new(i))
					.filter(move |i| !free.contains(&i))   )
    }
}
