use std::rc::{Rc};
use crate::mesh::Mesh;
use crate::mesh::ids::*;
use crate::mesh::connectivity_info::{HalfEdge, ConnectivityInfo};

/// # Traversal
impl Mesh
{
    pub fn walker(&self) -> Walker
    {
        Walker::new(&self.connectivity_info)
    }

    pub fn walker_from_vertex(&self, vertex_id: &VertexID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_vertex_halfedge_walker(vertex_id)
    }

    pub fn walker_from_halfedge(&self, halfedge_id: &HalfEdgeID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_halfedge_walker(halfedge_id)
    }

    pub fn walker_from_face(&self, face_id: &FaceID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_face_halfedge_walker(face_id)
    }
}

#[derive(Clone, Debug)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>
}

impl Walker
{
    pub(crate) fn new(connectivity_info: &Rc<ConnectivityInfo>) -> Self
    {
        Walker {current: None, current_info: None, connectivity_info: connectivity_info.clone()}
    }

    pub fn into_vertex_halfedge_walker(mut self, vertex_id: &VertexID) -> Self
    {
        self.as_vertex_halfedge_walker(vertex_id);
        self
    }

    pub fn as_vertex_halfedge_walker(&mut self, vertex_id: &VertexID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn into_halfedge_walker(mut self, halfedge_id: &HalfEdgeID) -> Self
    {
        self.as_halfedge_walker(halfedge_id);
        self
    }

    pub fn as_halfedge_walker(&mut self, halfedge_id: &HalfEdgeID) -> &mut Self
    {
        let halfedge_id = Some(halfedge_id.clone());
        self.set_current(halfedge_id);
        self
    }

    pub fn into_face_halfedge_walker(mut self, face_id: &FaceID) -> Self
    {
        self.as_face_halfedge_walker(face_id);
        self
    }

    pub fn as_face_halfedge_walker(&mut self, face_id: &FaceID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.face_halfedge(face_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn into_twin(mut self) -> Self
    {
        self.as_twin();
        self
    }

    pub fn as_twin(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.twin.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn twin_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.twin.clone() }
        else { None }
    }

    pub fn into_next(mut self) -> Self
    {
        self.as_next();
        self
    }

    pub fn as_next(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.next.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn next_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.next.clone() }
        else { None }
    }

    pub fn as_previous(&mut self) -> &mut Self
    {
        self.as_next().as_next()
    }

    pub fn into_previous(mut self) -> Self
    {
        self.as_next().as_next();
        self
    }

    pub fn previous_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref next_id) = self.next_id() { Walker::new(&self.connectivity_info.clone()).into_halfedge_walker(next_id).next_id() }
        else { None }
    }

    pub fn vertex_id(&self) -> Option<VertexID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.vertex.clone() }
        else { None }
    }

    pub fn halfedge_id(&self) -> Option<HalfEdgeID>
    {
        self.current.clone()
    }

    pub fn face_id(&self) -> Option<FaceID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.face.clone() }
        else { None }
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>)
    {
        self.current_info = if let Some(ref id) = halfedge_id { self.connectivity_info.halfedge(id) } else { None };
        self.current = halfedge_id;
    }
}
