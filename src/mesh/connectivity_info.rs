use std::cell::{RefCell};
use std::collections::HashMap;
use crate::mesh::ids::*;

#[derive(Clone, Debug)]
pub(crate) struct ConnectivityInfo {
    vertices: RefCell<HashMap<VertexID, Vertex>>,
    halfedges: RefCell<HashMap<HalfEdgeID, HalfEdge>>,
    faces: RefCell<HashMap<FaceID, Face>>
}

impl ConnectivityInfo {
    pub fn new(no_vertices: usize, no_faces: usize) -> ConnectivityInfo
    {
        ConnectivityInfo {
            vertices: RefCell::new(HashMap::with_capacity(no_vertices)),
            halfedges: RefCell::new(HashMap::with_capacity(4 * no_faces)),
            faces: RefCell::new(HashMap::with_capacity(no_faces))
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

    // Creates a face and the three internal halfedges and connects them to eachother and to the three given vertices
    pub fn create_face(&self, vertex_id1: &VertexID, vertex_id2: &VertexID, vertex_id3: &VertexID) -> FaceID
    {
        let id = self.new_face();

        // Create inner halfedges
        let halfedge1 = self.new_halfedge(Some(vertex_id2.clone()), None, Some(id.clone()));
        let halfedge3 = self.new_halfedge(Some(vertex_id1.clone()), Some(halfedge1.clone()), Some(id.clone()));
        let halfedge2 = self.new_halfedge(Some(vertex_id3.clone()), Some(halfedge3.clone()), Some(id.clone()));

        self.set_halfedge_next(&halfedge1, Some(halfedge2.clone()));

        self.set_vertex_halfedge(&vertex_id1, Some(halfedge1.clone()));
        self.set_vertex_halfedge(&vertex_id2, Some(halfedge2));
        self.set_vertex_halfedge(&vertex_id3, Some(halfedge3));

        self.set_face_halfedge(&id, halfedge1);

        id
    }

    pub fn new_vertex(&self) -> VertexID
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);

        let len = vertices.len();
        let mut id = VertexID::new(len);
        for i in len+1..std::usize::MAX {
            if !vertices.contains_key(&id) { break }
            id = VertexID::new(i);
        }

        vertices.insert(id.clone(), Vertex { halfedge: None });
        id
    }

    pub fn new_halfedge(&self, vertex: Option<VertexID>, next: Option<HalfEdgeID>, face: Option<FaceID>) -> HalfEdgeID
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);

        let len = halfedges.len();
        let mut id = HalfEdgeID::new(len);
        for i in len+1..std::usize::MAX {
            if !halfedges.contains_key(&id) { break }
            id = HalfEdgeID::new(i);
        }

        halfedges.insert(id.clone(), HalfEdge { vertex, twin: None, next, face });
        id
    }

    fn new_face(&self) -> FaceID
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);

        let len = faces.len();
        let mut id = FaceID::new(len);
        for i in len+1..std::usize::MAX {
            if !faces.contains_key(&id) { break }
            id = FaceID::new(i);
        }

        faces.insert(id.clone(), Face { halfedge: None });
        id
    }

    pub fn add_vertex(&self, vertex_id: VertexID, vertex: Vertex)
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);
        vertices.insert(vertex_id, vertex);
    }

    pub fn add_halfedge(&self, halfedge_id: HalfEdgeID, halfedge: HalfEdge)
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        halfedges.insert(halfedge_id, halfedge);
    }

    pub fn add_face(&self, face_id: FaceID, face: Face)
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);
        faces.insert(face_id, face);
    }

    pub fn remove_vertex(&self, vertex_id: &VertexID)
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);
        vertices.remove(vertex_id).unwrap();
    }

    pub fn remove_halfedge(&self, halfedge_id: &HalfEdgeID)
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        let halfedge = halfedges.remove(halfedge_id).unwrap();
        if halfedge.twin.is_some()
        {
            halfedges.get_mut(&halfedge.twin.unwrap()).unwrap().twin = None;
        }
    }

    pub fn remove_face(&self, face_id: &FaceID)
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);
        faces.remove(face_id).unwrap();
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: Option<HalfEdgeID>)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = val;
    }

    pub fn set_halfedge_next(&self, id: &HalfEdgeID, val: Option<HalfEdgeID>)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().next = val;
    }

    pub fn set_halfedge_twin(&self, id1: HalfEdgeID, id2: HalfEdgeID)
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
        halfedges.get_mut(&id1).unwrap().twin = Some(id2);
        halfedges.get_mut(&id2).unwrap().twin = Some(id1);
    }

    pub fn set_halfedge_vertex(&self, id: &HalfEdgeID, val: VertexID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().vertex = Some(val);
    }

    pub fn set_halfedge_face(&self, id: &HalfEdgeID, val: Option<FaceID>)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().face = val;
    }

    pub fn set_face_halfedge(&self, id: &FaceID, val: HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces).get_mut(id).unwrap().halfedge = Some(val);
    }

    pub fn vertex_iterator(&self) -> Box<Iterator<Item = VertexID>>
    {
        let vertices = RefCell::borrow(&self.vertices);
        let t: Vec<VertexID> = vertices.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn halfedge_iterator(&self) -> Box<Iterator<Item = HalfEdgeID>>
    {
        let halfedges = RefCell::borrow(&self.halfedges);
        let t: Vec<HalfEdgeID> = halfedges.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn face_iterator(&self) -> Box<Iterator<Item = FaceID>>
    {
        let faces = RefCell::borrow(&self.faces);
        let t: Vec<FaceID> = faces.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn vertex(&self, vertex_id: &VertexID) -> Option<Vertex>
    {
        RefCell::borrow(&self.vertices).get(vertex_id).and_then(|vertex| Some(vertex.clone()))
    }

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdge>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).and_then(|halfedge| Some(halfedge.clone()))
    }

    fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> Option<VertexID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().vertex.clone()
    }

    fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().twin.clone()
    }

    fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().next.clone()
    }

    fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> Option<FaceID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().face.clone()
    }

    pub fn face(&self, face_id: &FaceID) -> Option<Face>
    {
        RefCell::borrow(&self.faces).get(face_id).and_then(|face| Some(face.clone()))
    }

    pub fn face_halfedge(&self, face_id: &FaceID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.faces).get(face_id).unwrap().halfedge.clone()
    }
}

impl std::fmt::Display for ConnectivityInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** VERTICES: ****")?;
        let vertices = RefCell::borrow(&self.vertices);
        writeln!(f, "Count: {}", vertices.len())?;
        for (id, info) in vertices.iter() {
            writeln!(f, "{}: {:?}", id, info)?;
        }
        writeln!(f, "**** Halfedges: ****")?;
        let halfedges = RefCell::borrow(&self.halfedges);
        writeln!(f, "Count: {}", halfedges.len())?;
        for (id, info) in halfedges.iter() {
            writeln!(f, "{}: {:?}", id, info)?;
        }
        writeln!(f, "**** Faces: ****")?;
        let faces = RefCell::borrow(&self.faces);
        writeln!(f, "Count: {}", faces.len())?;
        for (id, info) in faces.iter() {
            writeln!(f, "{}: {:?}", id, info)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Vertex {
    pub halfedge: Option<HalfEdgeID>
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