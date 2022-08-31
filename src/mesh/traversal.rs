//! Defines a [Walker] for easy and efficient traversal of the mesh.
//! See [Mesh](crate::mesh::Mesh) for how to construct a walker.

use crate::mesh::connectivity_info::{ConnectivityInfo, HalfEdge};
use crate::mesh::*;

/// # Traversal
/// Methods to construct a [Walker] which is used for easy and efficient traversal of the mesh.
/// See [Walker] for more information and examples.
impl Mesh {
    /// Creates an 'empty' [Walker], ie. a walker that is associated with any half-edge.
    pub(super) fn walker(&self) -> Walker {
        Walker::new(&self.connectivity_info)
    }

    /// Creates a [Walker] at the half-edge pointed to by the given vertex.
    pub fn walker_from_vertex(&self, vertex_id: VertexID) -> Walker {
        self.walker().into_vertex_halfedge_walker(vertex_id)
    }

    /// Creates a [Walker] at the given half-edge.
    pub fn walker_from_halfedge(&self, halfedge_id: HalfEdgeID) -> Walker {
        self.walker().into_halfedge_walker(halfedge_id)
    }

    /// Creates a [Walker] at the half-edge pointed to by the given face.
    pub fn walker_from_face(&self, face_id: FaceID) -> Walker {
        self.walker().into_face_halfedge_walker(face_id)
    }
}

///
/// Used for easy and efficient traversal of the mesh.
/// See [Mesh](../struct.Mesh.html#traversal) for how to construct a walker
/// and the examples below for instructions on how to use a walker.
///
/// **Note:** If you walk outside the mesh at some point, no error will be returned,
/// instead, all methods to extract an ID will return `None`.
///
/// # Examples
///
/// ## \# 1
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// # let halfedge_id = mesh.halfedge_iter().next().unwrap();
/// // Find the id of the vertex pointed to by a half-edge.
/// let vertex_id = mesh.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
/// ```
///
/// ## \# 2
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// # let halfedge_id = mesh.halfedge_iter().next().unwrap();
/// let mut walker = mesh.walker_from_halfedge(halfedge_id);
/// // Walk around the three sides of a face..
/// let result_halfedge_id = walker.as_next().as_next().next_id().unwrap();
/// // .. ending up at the same half-edge
/// assert_eq!(halfedge_id, result_halfedge_id);
/// ```
/// ## \# 3
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// # let face_id = mesh.face_iter().next().unwrap();
/// // Find one neighbouring face to the given face
/// let neighbour_face_id = mesh.walker_from_face(face_id).into_twin().face_id().unwrap();
/// ```
///
/// ## \# 4
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// # let face_id = mesh.face_iter().next().unwrap();
/// // Find the circumference of a face
/// let mut walker = mesh.walker_from_face(face_id);
/// let mut circumference = mesh.edge_length(walker.halfedge_id().unwrap());
/// walker.as_next();
/// circumference += mesh.edge_length(walker.halfedge_id().unwrap());
/// circumference += mesh.edge_length(walker.next_id().unwrap());
/// ```
///
/// ## \# 5
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// # let halfedge_id = mesh.halfedge_iter().next().unwrap();
/// // Check if the half-edge is on the boundary of the mesh
/// let mut walker = mesh.walker_from_halfedge(halfedge_id);
/// let is_on_boundary = walker.face_id().is_none() || walker.as_twin().face_id().is_none();
/// # assert!(!is_on_boundary);
/// ```
///
/// ## \# 6
///
/// ```
/// # use tri_mesh::*;
/// # let mesh: Mesh = RawMesh::sphere(4).into();
/// // Compute the average edge length
/// let mut avg_edge_length = 0.0f64;
/// for halfedge_id in mesh.edge_iter()
/// {
///     let mut walker = mesh.walker_from_halfedge(halfedge_id);
///     let p0 = mesh.vertex_position(walker.vertex_id().unwrap());
///     let p1 = mesh.vertex_position(walker.as_twin().vertex_id().unwrap());
///     avg_edge_length += (p0 - p1).magnitude();
/// }
/// avg_edge_length /= mesh.no_edges() as f64;
/// ```
///
#[derive(Clone, Debug)]
pub struct Walker<'a> {
    connectivity_info: &'a ConnectivityInfo,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>,
}

impl<'a> Walker<'a> {
    pub(super) fn new(connectivity_info: &'a ConnectivityInfo) -> Self {
        Walker {
            current: None,
            current_info: None,
            connectivity_info: connectivity_info,
        }
    }

    /// Jumps to the half-edge pointed to by the given vertex.
    pub(super) fn into_vertex_halfedge_walker(mut self, vertex_id: VertexID) -> Self {
        self.as_vertex_halfedge_walker(vertex_id);
        self
    }

    /// Jumps to the given half-edge.
    pub(super) fn into_halfedge_walker(mut self, halfedge_id: HalfEdgeID) -> Self {
        self.as_halfedge_walker(halfedge_id);
        self
    }

    /// Jumps to the half-edge pointed to by the given face.
    pub(super) fn into_face_halfedge_walker(mut self, face_id: FaceID) -> Self {
        self.as_face_halfedge_walker(face_id);
        self
    }

    /// Jumps to the half-edge pointed to by the given vertex.
    pub(super) fn as_vertex_halfedge_walker(&mut self, vertex_id: VertexID) -> &mut Self {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    /// Jumps to the given half-edge.
    pub(super) fn as_halfedge_walker(&mut self, halfedge_id: HalfEdgeID) -> &mut Self {
        let halfedge_id = Some(halfedge_id);
        self.set_current(halfedge_id);
        self
    }

    /// Jumps to the half-edge pointed to by the given face.
    pub(super) fn as_face_halfedge_walker(&mut self, face_id: FaceID) -> &mut Self {
        let halfedge_id = self.connectivity_info.face_halfedge(face_id);
        self.set_current(halfedge_id);
        self
    }

    /// Walk to the next half-edge in the adjacent face.
    pub fn into_next(mut self) -> Self {
        self.as_next();
        self
    }

    /// Walk to the previous half-edge in the adjacent face.
    pub fn into_previous(mut self) -> Self {
        self.as_next().as_next();
        self
    }

    /// Walk to the twin half-edge.
    pub fn into_twin(mut self) -> Self {
        self.as_twin();
        self
    }

    /// Walk to the next half-edge in the adjacent face.
    pub fn as_next(&mut self) -> &mut Self {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => current_info.next,
            None => None,
        };
        self.set_current(halfedge_id);
        self
    }

    /// Walk to the previous half-edge in the adjacent face.
    pub fn as_previous(&mut self) -> &mut Self {
        self.as_next().as_next()
    }

    /// Walk to the twin half-edge.
    pub fn as_twin(&mut self) -> &mut Self {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => current_info.twin,
            None => None,
        };
        self.set_current(halfedge_id);
        self
    }

    /// Returns the id of the vertex pointed to by the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn vertex_id(&self) -> Option<VertexID> {
        if let Some(ref halfedge) = self.current_info {
            halfedge.vertex
        } else {
            None
        }
    }

    /// Returns the id of the next half-edge in the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn next_id(&self) -> Option<HalfEdgeID> {
        if let Some(ref halfedge) = self.current_info {
            halfedge.next
        } else {
            None
        }
    }

    /// Returns the id of the previous half-edge in the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn previous_id(&self) -> Option<HalfEdgeID> {
        if let Some(next_id) = self.next_id() {
            Walker::new(&self.connectivity_info)
                .into_halfedge_walker(next_id)
                .next_id()
        } else {
            None
        }
    }

    /// Returns the id of the twin half-edge to the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn twin_id(&self) -> Option<HalfEdgeID> {
        if let Some(ref halfedge) = self.current_info {
            halfedge.twin
        } else {
            None
        }
    }

    /// Returns the id of the current half-edge or `None` if the walker has walked outside of the mesh at some point.
    pub fn halfedge_id(&self) -> Option<HalfEdgeID> {
        self.current
    }

    /// Returns the id of the adjacent face or `None` if the half-edge is at the boundary of the mesh
    /// or if the walker has walked outside of the mesh at some point.
    pub fn face_id(&self) -> Option<FaceID> {
        if let Some(ref halfedge) = self.current_info {
            halfedge.face
        } else {
            None
        }
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>) {
        self.current_info = if let Some(id) = halfedge_id {
            self.connectivity_info.halfedge(id)
        } else {
            None
        };
        self.current = halfedge_id;
    }
}
