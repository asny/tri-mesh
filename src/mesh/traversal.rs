//! See [Mesh](crate::mesh::Mesh).

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

    pub fn walker_from_vertex(&self, vertex_id: VertexID) -> Walker
    {
        self.walker().into_vertex_halfedge_walker(vertex_id)
    }

    pub fn walker_from_halfedge(&self, halfedge_id: HalfEdgeID) -> Walker
    {
        self.walker().into_halfedge_walker(halfedge_id)
    }

    pub fn walker_from_face(&self, face_id: FaceID) -> Walker
    {
        self.walker().into_face_halfedge_walker(face_id)
    }
}

#[derive(Clone, Debug)]
pub struct Walker<'a>
{
    connectivity_info: &'a ConnectivityInfo,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>
}

impl<'a> Walker<'a>
{
    pub(crate) fn new(connectivity_info: &'a ConnectivityInfo) -> Self
    {
        Walker {current: None, current_info: None, connectivity_info: connectivity_info}
    }

    pub fn into_vertex_halfedge_walker(mut self, vertex_id: VertexID) -> Self
    {
        self.as_vertex_halfedge_walker(vertex_id);
        self
    }

    pub fn into_halfedge_walker(mut self, halfedge_id: HalfEdgeID) -> Self
    {
        self.as_halfedge_walker(halfedge_id);
        self
    }

    pub fn into_face_halfedge_walker(mut self, face_id: FaceID) -> Self
    {
        self.as_face_halfedge_walker(face_id);
        self
    }

    pub fn as_vertex_halfedge_walker(&mut self, vertex_id: VertexID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn as_halfedge_walker(&mut self, halfedge_id: HalfEdgeID) -> &mut Self
    {
        let halfedge_id = Some(halfedge_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn as_face_halfedge_walker(&mut self, face_id: FaceID) -> &mut Self
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

    pub fn into_next(mut self) -> Self
    {
        self.as_next();
        self
    }

    pub fn into_previous(mut self) -> Self
    {
        self.as_next().as_next();
        self
    }

    pub fn as_next(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.next },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn as_previous(&mut self) -> &mut Self
    {
        self.as_next().as_next()
    }

    pub fn as_twin(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.twin },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    /// Returns the id of the vertex pointed to by the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn vertex_id(&self) -> Option<VertexID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.vertex }
        else { None }
    }

    /// Returns the id of the next half-edge in the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn next_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.next }
        else { None }
    }

    /// Returns the id of the previous half-edge in the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn previous_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(next_id) = self.next_id() { Walker::new(&self.connectivity_info).into_halfedge_walker(next_id).next_id() }
        else { None }
    }

    /// Returns the id of the twin half-edge to the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn twin_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.twin }
        else { None }
    }

    /// Returns the id of the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn halfedge_id(&self) -> Option<HalfEdgeID>
    {
        self.current
    }

    /// Returns the id of the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn face_id(&self) -> Option<FaceID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.face }
        else { None }
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>)
    {
        self.current_info = if let Some(id) = halfedge_id { self.connectivity_info.halfedge(id) } else { None };
        self.current = halfedge_id;
    }
}
