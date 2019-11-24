//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use crate::mesh::math::*;
use crate::mesh::ids::*;

/// # Edit
impl Mesh
{
    /// Flip the given edge such that the edge after the flip is connected to the
    /// other pair of the four vertices connected to the two adjacent faces.
    ///
    /// ```text
    ///   /\          /|\
    ///  /  \        / | \
    /// /____\  --> /  |  \
    /// \    /      \  |  /
    ///  \  /        \ | /
    ///   \/          \|/
    /// ```
    ///
    /// # Error
    ///
    /// Returns an error if trying to flip an edge on the boundary or the flip will connect two vertices that are already connected by another edge.
    ///
    pub fn flip_edge(&mut self, halfedge_id: HalfEdgeID) -> Result<(), Error>
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let face_id = walker.face_id().ok_or(Error::ActionWillResultInInvalidMesh {message: format!("Trying to flip edge on boundary")})?;
        let next_id = walker.next_id().unwrap();
        let previous_id = walker.previous_id().unwrap();
        let v0 = walker.vertex_id().unwrap();
        walker.as_next();
        let v3 = walker.vertex_id().unwrap();
        walker.as_previous();

        walker.as_twin();
        let twin_id = walker.halfedge_id().unwrap();
        let twin_face_id = walker.face_id().ok_or(Error::ActionWillResultInInvalidMesh {message: format!("Trying to flip edge on boundary")})?;
        let twin_next_id = walker.next_id().unwrap();
        let twin_previous_id = walker.previous_id().unwrap();
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.as_next().vertex_id().unwrap();

        if self.connecting_edge(v2, v3).is_some() { return Err(Error::ActionWillResultInInvalidMesh {message: format!("Trying to flip edge which will connect two vertices that are already connected by another edge")}) }

        self.connectivity_info.set_face_halfedge(face_id, previous_id);
        self.connectivity_info.set_face_halfedge(twin_face_id, twin_previous_id);

        self.connectivity_info.set_vertex_halfedge(v0, Some(next_id));
        self.connectivity_info.set_vertex_halfedge(v1, Some(twin_next_id));

        self.connectivity_info.set_halfedge_next(halfedge_id, Some(previous_id));
        self.connectivity_info.set_halfedge_next(next_id, Some(twin_id));
        self.connectivity_info.set_halfedge_next(previous_id, Some(twin_next_id));
        self.connectivity_info.set_halfedge_next(twin_id, Some(twin_previous_id));
        self.connectivity_info.set_halfedge_next(twin_next_id, Some(halfedge_id));
        self.connectivity_info.set_halfedge_next(twin_previous_id, Some(next_id));

        self.connectivity_info.set_halfedge_vertex(halfedge_id, v3);
        self.connectivity_info.set_halfedge_vertex(twin_id, v2);

        self.connectivity_info.set_halfedge_face(next_id, Some(twin_face_id));
        self.connectivity_info.set_halfedge_face(twin_next_id, Some(face_id));

        Ok(())
    }
    
	/// split a vertex in two vertices, along the edges `start` and `end`, returning the created point.
	/// - `start` and `end` must point to the same vertex.
	/// - the two points remains in the same initial location.
	/// - the point created becomes the vertex pointed by `start`
	/// ```
	///        +                           +
	///        |                          / \
	///  ------+-------     -->      ----+   +----
	///        |                          \ /
	///        +                           +
	/// ```
	pub fn split_vertex(&mut self, start: HalfEdgeID, end: HalfEdgeID) -> VertexID {
        // get start vertex of `start` and `end` and their twin
        let (vstart, rstart) = {
            let walker = self.walker_from_halfedge(start).into_twin();
            (walker.vertex_id().unwrap(), walker.halfedge_id().unwrap())
            };
        let (vend, rend) = {
            let walker = self.walker_from_halfedge(end).into_twin();
            (walker.vertex_id().unwrap(), walker.halfedge_id().unwrap())
            };
        // splited vertex
        let old = self.walker_from_halfedge(start).vertex_id().unwrap();
        let created = self.split_vertex_unfinished(start, end);
        
        // create twins for the separated halfedges
        let conn = &mut self.connectivity_info;
        conn.set_halfedge_twin(start,   conn.new_halfedge(Some(vstart),     None, None));
        conn.set_halfedge_twin(rstart,  conn.new_halfedge(Some(old),        None, None));
        conn.set_halfedge_twin(end,     conn.new_halfedge(Some(vend),       None, None));
        conn.set_halfedge_twin(rend,    conn.new_halfedge(Some(created),    None, None));
        created
	}
	fn split_vertex_unfinished(&mut self, start: HalfEdgeID, end: HalfEdgeID) -> VertexID {
		assert_eq!(
			self.walker_from_halfedge(start).vertex_id().unwrap(), 
			self.walker_from_halfedge(end).vertex_id().unwrap(),
			"spliting halfedges doesn't point to the same vertex");
		
		// duplicate the vertex
		let newvert = self.connectivity_info.new_vertex(self.vertex_position(self.walker_from_halfedge(start).vertex_id().unwrap()));
		self.connectivity_info.set_vertex_halfedge(newvert, self.walker_from_halfedge(end).as_twin().halfedge_id());
		// cut at start and at end
		self.connectivity_info.remove_halfedge_twin(start);
		self.connectivity_info.remove_halfedge_twin(end);
		// change refs to old vertex between these two halfedges
		let mut walker = self.walker_from_halfedge(start);
		while let Some(he) = walker.halfedge_id() {
			self.connectivity_info.set_halfedge_vertex(he, newvert);
			walker.as_next().as_twin();
		}
		
		newvert
	}

    /// Split the given edge into two.
    /// Returns the id of the new vertex positioned at the given position.
    pub fn split_edge(&mut self, halfedge_id: HalfEdgeID, position: Vec3) -> VertexID
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        if walker.face_id().is_none()
        {
            walker.as_twin();
        }
        let split_halfedge_id = walker.halfedge_id().unwrap();

        walker.as_twin();
        let twin_halfedge_id = walker.halfedge_id().unwrap();
        let twin_vertex_id = walker.vertex_id();
        let is_boundary = walker.face_id().is_none();

        let new_vertex_id = self.create_vertex(position);
        self.split_one_face(split_halfedge_id, twin_halfedge_id, new_vertex_id);

        if !is_boundary {
            self.split_one_face(twin_halfedge_id, split_halfedge_id, new_vertex_id);
        }
        else {
            let new_halfedge_id = self.connectivity_info.new_halfedge(twin_vertex_id, None, None);
            self.connectivity_info.set_halfedge_twin(split_halfedge_id, new_halfedge_id);
            self.connectivity_info.set_halfedge_vertex(twin_halfedge_id, new_vertex_id);
        };

        new_vertex_id
    }

    /// Split the given face into three new faces.
    /// Returns the id of the new vertex positioned at the given position.
    pub fn split_face(&mut self, face_id: FaceID, position: Vec3) -> VertexID
    {
        let new_vertex_id = self.create_vertex(position);

        let mut walker = self.walker_from_face(face_id);
        let halfedge_id1 = walker.halfedge_id().unwrap();
        let vertex_id1 = walker.vertex_id().unwrap();

        walker.as_next();
        let halfedge_id2 = walker.halfedge_id().unwrap();
        let vertex_id2 = walker.vertex_id().unwrap();

        walker.as_next();
        let halfedge_id3 = walker.halfedge_id().unwrap();
        let vertex_id3 = walker.vertex_id().unwrap();

        let face_id1 = self.connectivity_info.create_face_with_existing_halfedge(vertex_id1, vertex_id2, new_vertex_id, halfedge_id2);
        let face_id2 = self.connectivity_info.create_face_with_existing_halfedge(vertex_id2, vertex_id3, new_vertex_id, halfedge_id3);

        let new_halfedge_id2 = self.connectivity_info.new_halfedge(Some(vertex_id3), Some(halfedge_id1), Some(face_id));
        let new_halfedge_id1 = self.connectivity_info.new_halfedge(Some(new_vertex_id), Some(new_halfedge_id2), Some(face_id));
        self.connectivity_info.set_halfedge_next(halfedge_id1, Some(new_halfedge_id1));
        self.connectivity_info.set_face_halfedge(face_id, halfedge_id1);

        // Update twin information
        let mut new_halfedge_id = HalfEdgeID::new(0);
        for halfedge_id in self.face_halfedge_iter(face_id1) {
            let vid = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
            if vid == vertex_id1 {
                self.connectivity_info.set_halfedge_twin(new_halfedge_id1, halfedge_id);
            }
            else if vid == new_vertex_id {
                new_halfedge_id = halfedge_id;
            }
        }
        for halfedge_id in self.face_halfedge_iter(face_id2) {
            let vid = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
            if vid == vertex_id2 {
                self.connectivity_info.set_halfedge_twin(new_halfedge_id, halfedge_id);
            }
            else if vid == new_vertex_id {
                self.connectivity_info.set_halfedge_twin(new_halfedge_id2, halfedge_id);
            }
        }
        new_vertex_id
    }

    fn split_one_face(&mut self, halfedge_id: HalfEdgeID, twin_halfedge_id: HalfEdgeID, new_vertex_id: VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let vertex_id1 = walker.vertex_id().unwrap();
        let old_face_id = walker.face_id().unwrap();

        walker.as_next();
        let halfedge_to_reuse_vertex = walker.vertex_id().unwrap();
        let halfedge_to_reuse = walker.halfedge_id().unwrap();
        let halfedge_to_reuse_next = walker.next_id().unwrap();

        // Create new face
        let new_face_id = self.connectivity_info.create_face_with_existing_halfedge(vertex_id1, halfedge_to_reuse_vertex, new_vertex_id, halfedge_to_reuse);

        // Update old face
        let new_halfedge_id = self.connectivity_info.new_halfedge(Some(halfedge_to_reuse_vertex), Some(halfedge_to_reuse_next), Some(old_face_id));
        self.connectivity_info.set_halfedge_vertex(halfedge_id, new_vertex_id);
        self.connectivity_info.set_halfedge_next(halfedge_id, Some(new_halfedge_id));
        self.connectivity_info.set_face_halfedge(old_face_id, halfedge_id);

        // Update twin information
        for halfedge_id in self.face_halfedge_iter(new_face_id) {
            let vid = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
            if vid == vertex_id1 {
                self.connectivity_info.set_halfedge_twin(twin_halfedge_id, halfedge_id);
            }
            else if vid == new_vertex_id {
                self.connectivity_info.set_halfedge_twin(new_halfedge_id, halfedge_id);
            }
        }
    }

    ///
    /// Collapses the given edge. Consequently, the to adjacent faces are removed and
    /// the two adjacent vertices are merged into one vertex
    /// which position is the average of the original vertex positions.
    /// Returns the merged vertex.
    ///
    /// **Note:** This might make some faces degenerate.
    ///
    pub fn collapse_edge(&mut self, halfedge_id: HalfEdgeID) -> VertexID
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let surviving_vertex_id = walker.vertex_id().unwrap();
        walker.as_twin();
        let dying_vertex_id = walker.vertex_id().unwrap();
        let new_position = 0.5 * (self.vertex_position(surviving_vertex_id) + self.vertex_position(dying_vertex_id));


        // Update halfedges pointing to dying vertex
        for halfedge_id in self.vertex_halfedge_iter(dying_vertex_id) {
            self.connectivity_info.set_halfedge_vertex(self.walker_from_halfedge(halfedge_id).twin_id().unwrap(), surviving_vertex_id);
        }

        // Remove first face + halfedges
        let mut he_id1 = walker.halfedge_id();
        if walker.face_id().is_some() {
            walker.as_previous();
            self.connectivity_info.set_vertex_halfedge( surviving_vertex_id, walker.twin_id());
            walker.as_next();
        }
        else {
            self.connectivity_info.remove_halfedge(he_id1.unwrap());
            he_id1 = None;
        }

        // Remove second face + halfedges
        walker.as_twin();
        let he_id2 = walker.halfedge_id().unwrap();
        if walker.face_id().is_some() {
            walker.as_previous();
            self.connectivity_info.set_vertex_halfedge( surviving_vertex_id, walker.twin_id());
            self.remove_one_face(he_id2);
        }
        else {
            self.connectivity_info.remove_halfedge(he_id2);
        }

        if let Some(he_id_to_remove) = he_id1 {
            self.remove_one_face(he_id_to_remove);
        }

        // Remove dying vertex
        self.connectivity_info.remove_vertex(dying_vertex_id);

        self.move_vertex_to(surviving_vertex_id, new_position);
        surviving_vertex_id
    }

    fn remove_one_face(&mut self, halfedge_id: HalfEdgeID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let face_id = walker.face_id().unwrap();

        walker.as_next();
        let halfedge_id1 = walker.halfedge_id().unwrap();
        let twin_id1 = walker.twin_id().unwrap();
        let vertex_id = walker.vertex_id().unwrap();
        walker.as_next();
        let halfedge_id2 = walker.halfedge_id().unwrap();
        let twin_id2 = walker.twin_id().unwrap();

        self.connectivity_info.remove_face(face_id);
        self.connectivity_info.remove_halfedge(halfedge_id);
        self.connectivity_info.remove_halfedge(halfedge_id1);
        self.connectivity_info.remove_halfedge(halfedge_id2);
        self.connectivity_info.set_halfedge_twin(twin_id1, twin_id2);
        self.connectivity_info.set_vertex_halfedge(vertex_id, Some(twin_id1));

        walker.as_twin();

    }

    /// Removes the given face and the adjacent edges if they are then not connected to any face.
    pub fn remove_face(&mut self, face_id: FaceID)
    {
        let edges: Vec<HalfEdgeID> = self.face_halfedge_iter(face_id).collect();
        self.remove_face_unsafe(face_id);
        for halfedge_id in edges {
            self.remove_edge_if_lonely(halfedge_id);
        }
    }

    pub(super) fn remove_edge_if_lonely(&mut self, halfedge_id: HalfEdgeID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        if walker.face_id().is_none() && walker.as_twin().face_id().is_none()
        {
            let vertex_id1 = walker.vertex_id().unwrap();
            let halfedge_id1 = walker.halfedge_id().unwrap();
            walker.as_twin();
            let vertex_id2 = walker.vertex_id().unwrap();
            let halfedge_id2 = walker.halfedge_id().unwrap();

            self.connectivity_info.remove_halfedge(halfedge_id1);
            self.connectivity_info.remove_halfedge(halfedge_id2);

            let find_new_edge_connectivity = |mesh: &Mesh, vertex_id: VertexID| -> Option<HalfEdgeID>
            {
                for halfedge_id in mesh.halfedge_iter() {
                    let walker = mesh.walker_from_halfedge(halfedge_id);
                    if walker.vertex_id().unwrap() == vertex_id {
                        return walker.twin_id();
                    }
                }
                None
            };

            if self.walker_from_vertex(vertex_id1).halfedge_id().unwrap() == halfedge_id2 {
                let new_edge = find_new_edge_connectivity(&self, vertex_id1);
                self.connectivity_info.set_vertex_halfedge(vertex_id1, new_edge);
                self.remove_vertex_if_lonely(vertex_id1);
            }
            if self.walker_from_vertex(vertex_id2).halfedge_id().unwrap() == halfedge_id1 {
                let new_edge = find_new_edge_connectivity(&self, vertex_id2);
                self.connectivity_info.set_vertex_halfedge(vertex_id2, new_edge);
                self.remove_vertex_if_lonely(vertex_id2);
            }
        }
    }

    pub(super) fn remove_vertex_if_lonely(&mut self, vertex_id: VertexID)
    {
        if self.connectivity_info.vertex_halfedge(vertex_id).is_none()
        {
            self.connectivity_info.remove_vertex(vertex_id);
        }
    }

    pub(super) fn remove_face_unsafe(&mut self, face_id: FaceID)
    {
        let mut walker = self.walker_from_face(face_id);
        let he_id1 = walker.halfedge_id().unwrap();
        walker.as_next();
        let he_id2 = walker.halfedge_id().unwrap();
        walker.as_next();
        let he_id3 = walker.halfedge_id().unwrap();

        self.connectivity_info.set_halfedge_face(he_id1, None);
        self.connectivity_info.set_halfedge_face(he_id2, None);
        self.connectivity_info.set_halfedge_face(he_id3, None);

        self.connectivity_info.set_halfedge_next(he_id1, None);
        self.connectivity_info.set_halfedge_next(he_id2, None);
        self.connectivity_info.set_halfedge_next(he_id3, None);

        self.connectivity_info.remove_face(face_id);
    }
    
    
    /// Create a bevel on the given edges, these must be contiguous and ordered (ie. `edge[i]` starts from `edge[i-1]`)
    /// This function is not a realistic chamfer, it can displace the edges that are cutted by the bevel.
    ///
    pub fn bevel_curve(&mut self, edge: &[VertexID], amount: f64)
    {
        assert!(edge.len() >= 2, "`edge` must be at least two points");
        
        let err_continuous = "edge must be a list of contiguous vertices";
        let err_border = "edge must not be on a border of the mesh";
        
        // get the start halfedge (before the first vertex of the edge)
        let rfirst = next_forward(self, self.connecting_edge(edge[1], edge[0]) .expect(err_continuous)) .expect(err_border);
        let startvert = self.walker_from_halfedge(rfirst).vertex_id().unwrap();
        let first = self.walker_from_halfedge(rfirst).as_twin().halfedge_id().unwrap();
        // get the end halfedge (after the last vertex of the edge)
        let last = next_forward(self, self.connecting_edge(edge[edge.len()-2], edge[edge.len()-1]) .expect(err_continuous)) .expect(err_border);
        let endvert = self.walker_from_halfedge(last).vertex_id().unwrap();
        
        // get all the informations for the operation, since while modifying the mesh, the halfedge functions doesn't work
        let mut infos = Vec::with_capacity(edge.len());
        
        let mut previous = first;
        let mut next = first;
        for i in 0 .. edge.len() {
            // get the next edge to bevel else the ending one
            if i < edge.len()-1		{ next = self.connecting_edge(edge[i+1], edge[i]) .expect(err_continuous); }
            else 					{ next = last; }
            
            let d1 = (	self.face_normal(self.walker_from_halfedge(previous).face_id().unwrap()) 
                            .cross(self.edge_direction(previous))
                        -	self.face_normal(self.walker_from_halfedge(next).as_twin().face_id().unwrap()) 
                            .cross(self.edge_direction(next))
                        ).normalize() * amount;
            
            let d2 = (	self.face_normal(self.walker_from_halfedge(previous).as_twin().face_id().unwrap()) 
                            .cross(self.edge_direction(previous))
                        +	self.face_normal(self.walker_from_halfedge(next).face_id().unwrap()) 
                            .cross(self.edge_direction(next))
                        ) .normalize() * amount;
            
            infos.push(([d1, d2], [last,next]));
            
            previous = next;
        }
        
        let mut facing = Vec::with_capacity(edge.len());
        
        previous = first;
        next = first;
        for i in 0 .. edge.len() {
            // get the next edge to bevel else the ending one
            if i < edge.len()-1		{ next = self.connecting_edge(edge[i+1], edge[i]) .expect(err_continuous); }
            else 					{ next = last; }
            
            // separate the vertex in two vertices
            let vert1 = edge[i];
            let vert2 = self.split_vertex_unfinished(last, self.walker_from_halfedge(next).as_twin().halfedge_id().unwrap());
            facing.push([vert1, vert2]);
            // displace points						
            self.connectivity_info.set_position(vert1, self.vertex_position(vert1) + infos[i].0[0]);
            self.connectivity_info.set_position(vert2, self.vertex_position(vert2) + infos[i].0[1]);
                        
            previous = next;
        }
        
        // create start and end face
        self.connectivity_info.create_face(startvert, facing[0][0], facing[0][1]);
        self.connectivity_info.create_face(endvert, facing.last().unwrap()[1], facing.last().unwrap()[0]);
        // create all faces
        for confront in facing.windows(2) {
            self.connectivity_info.create_face(confront[0][0], confront[0][1], confront[1][1]);
            self.connectivity_info.create_face(confront[1][1], confront[1][0], confront[0][0]);
        }
        self.twin_alones();
    }
}


/// return the halfedge starting from `he`'s vertex, that has the closest direction to `he`
fn next_forward(mesh: &Mesh, start: HalfEdgeID) -> Option<HalfEdgeID> {
    // nominal direction, the direction of he
    let nominal = mesh.edge_direction(start);
    let mut score = -1.;
    let mut next = None;
    
    for he in mesh.vertex_halfedge_iter(mesh.walker_from_halfedge(start).vertex_id().unwrap()) {
        let s = mesh.edge_direction(he) .dot(nominal);
        if s > score {
            next = Some(he);
            score = s;
        }
    }
    next
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_builder::MeshBuilder;

    #[test]
    fn test_flip_edge()
    {
        let mut no_flips = 0;
        let mut mesh = MeshBuilder::new().square().build().unwrap();
        let no_edges = mesh.no_halfedges();
        for halfedge_id in mesh.halfedge_iter() {
            let (v0, v1) = mesh.edge_vertices(halfedge_id);

            if mesh.flip_edge(halfedge_id).is_ok()
            {
                mesh.is_valid().unwrap();

                let (v2, v3) = mesh.edge_vertices(halfedge_id);
                assert_ne!(v0, v2);
                assert_ne!(v1, v2);
                assert_ne!(v0, v3);
                assert_ne!(v1, v3);

                assert!(mesh.connecting_edge(v0, v1).is_none());
                assert!(mesh.connecting_edge(v2, v3).is_some());

                let edge = mesh.connecting_edge(v2, v3).unwrap();
                let twin = mesh.walker_from_halfedge(edge).twin_id().unwrap();
                assert!(edge == halfedge_id || twin == halfedge_id,
                        format!("Flipped edge {} or flipped edge twin {} should be equal to before flipped edge id {}", edge, twin, halfedge_id));
                no_flips = no_flips + 1;
            }
        }
        assert_eq!(no_edges, mesh.no_halfedges());
        assert_eq!(no_flips, 2);
    }

    #[test]
    fn test_flip_multiple_edges()
    {
        let mut no_flips = 0;
        let mut mesh = MeshBuilder::new().icosahedron().build().unwrap();
        let no_edges = mesh.no_halfedges();
        for halfedge_id in mesh.halfedge_iter() {
            let (v0, v1) = mesh.edge_vertices(halfedge_id);

            if mesh.flip_edge(halfedge_id).is_ok()
            {
                mesh.is_valid().unwrap();

                let (v2, v3) = mesh.edge_vertices(halfedge_id);
                assert_ne!(v0, v2);
                assert_ne!(v1, v2);
                assert_ne!(v0, v3);
                assert_ne!(v1, v3);

                assert!(mesh.connecting_edge(v0, v1).is_none());
                assert!(mesh.connecting_edge(v2, v3).is_some());

                let edge = mesh.connecting_edge(v2, v3).unwrap();
                let twin = mesh.walker_from_halfedge(edge).twin_id().unwrap();
                assert!(edge == halfedge_id || twin == halfedge_id,
                        format!("Flipped edge {} or flipped edge twin {} should be equal to before flipped edge id {}", edge, twin, halfedge_id));
                no_flips = no_flips + 1;
            }
        }
        assert_eq!(no_edges, mesh.no_halfedges());
        assert!(no_flips > 0);
    }

    #[test]
    fn test_split_edge_on_boundary()
    {
        let mut mesh = MeshBuilder::new().triangle().build().unwrap();
        for halfedge_id in mesh.halfedge_iter()
        {
            if mesh.walker_from_halfedge(halfedge_id).face_id().is_some()
            {
                mesh.split_edge(halfedge_id, vec3(-1.0, -1.0, -1.0));

                assert_eq!(mesh.no_vertices(), 4);
                assert_eq!(mesh.no_halfedges(), 2 * 3 + 4);
                assert_eq!(mesh.no_faces(), 2);

                let mut walker = mesh.walker_from_halfedge(halfedge_id);
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_some());
                assert!(walker.vertex_id().is_some());

                walker.as_twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_none());
                assert!(walker.vertex_id().is_some());

                walker.as_twin().as_next().as_twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_some());
                assert!(walker.vertex_id().is_some());

                walker.as_next().as_next().as_twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_none());
                assert!(walker.vertex_id().is_some());

                mesh.is_valid().unwrap();

                break;
            }
        }
    }

    #[test]
    fn test_split_edge()
    {
        let mut mesh = MeshBuilder::new().square().build().unwrap();
        for halfedge_id in mesh.halfedge_iter() {
            let mut walker = mesh.walker_from_halfedge(halfedge_id);
            if walker.face_id().is_some() && walker.as_twin().face_id().is_some()
            {
                let vertex_id = mesh.split_edge(halfedge_id, vec3(-1.0, -1.0, -1.0));
                assert_eq!(mesh.no_vertices(), 5);
                assert_eq!(mesh.no_halfedges(), 4 * 3 + 4);
                assert_eq!(mesh.no_faces(), 4);

                let mut w = mesh.walker_from_vertex(vertex_id);
                let start_halfedge_id = w.halfedge_id();
                let mut end_halfedge_id = w.twin_id();
                for _ in 0..4 {
                    assert!(w.halfedge_id().is_some());
                    assert!(w.twin_id().is_some());
                    assert!(w.vertex_id().is_some());
                    assert!(w.face_id().is_some());
                    w.as_previous().as_twin();
                    end_halfedge_id = w.halfedge_id();
                }
                assert_eq!(start_halfedge_id, end_halfedge_id, "Did not go the full round");

                mesh.is_valid().unwrap();
                break;
            }
        }
    }

    #[test]
    fn test_split_face()
    {
        let mut mesh = MeshBuilder::new().triangle().build().unwrap();
        let face_id = mesh.face_iter().next().unwrap();

        let vertex_id = mesh.split_face(face_id, vec3(-1.0, -1.0, -1.0));

        assert_eq!(mesh.no_vertices(), 4);
        assert_eq!(mesh.no_halfedges(), 3 * 3 + 3);
        assert_eq!(mesh.no_faces(), 3);

        let mut walker = mesh.walker_from_vertex(vertex_id);
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.as_previous().as_twin().as_previous().as_twin().as_previous().as_twin().halfedge_id().unwrap();
        assert_eq!(start_edge, one_round_edge);

        assert!(walker.face_id().is_some());
        walker.as_next().as_twin();
        assert!(walker.face_id().is_none());

        walker.as_twin().as_next().as_twin().as_next().as_twin();
        assert!(walker.face_id().is_none());

        walker.as_twin().as_next().as_twin().as_next().as_twin();
        assert!(walker.face_id().is_none());

        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_collapse_edge_on_boundary1()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  2, 3, 4  ];
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0,  1.0, 0.0, 1.0,  2.0, 0.0, 0.5];
        let mut mesh = Mesh::new(indices, positions);

        for halfedge_id in mesh.halfedge_iter()
        {
            let mut walker = mesh.walker_from_halfedge(halfedge_id);
            if walker.face_id().is_none() && walker.as_twin().as_next().as_twin().face_id().is_some() && walker.as_twin().as_next().as_twin().face_id().is_some()
            {
                mesh.collapse_edge(halfedge_id);

                assert_eq!(mesh.no_vertices(), 4);
                assert_eq!(mesh.no_halfedges(), 10);
                assert_eq!(mesh.no_faces(), 2);

                mesh.is_valid().unwrap();

                break;
            }
        }
    }

    #[test]
    fn test_collapse_edge_on_boundary2()
    {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1];
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0,  1.0, 0.0, 1.0];
        let mut mesh = Mesh::new(indices, positions);
        for halfedge_id in mesh.halfedge_iter()
        {
            if mesh.is_edge_on_boundary(halfedge_id)
            {
                mesh.collapse_edge(halfedge_id);

                assert_eq!(mesh.no_vertices(), 3);
                assert_eq!(mesh.no_halfedges(), 6);
                assert_eq!(mesh.no_faces(), 1);


                mesh.is_valid().unwrap();

                break;
            }
        }
    }

    #[test]
    fn test_collapse_edge()
    {
        let mut mesh = MeshBuilder::new().subdivided_triangle().build().unwrap();
        for halfedge_id in mesh.halfedge_iter() {
            if !mesh.is_edge_on_boundary(halfedge_id)
            {
                mesh.collapse_edge(halfedge_id);
                assert_eq!(mesh.no_vertices(), 3);
                assert_eq!(mesh.no_halfedges(), 6);
                assert_eq!(mesh.no_faces(), 1);

                mesh.is_valid().unwrap();
                break;
            }
        }
    }

    #[test]
    fn test_recursive_collapse_edge()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  2, 3, 4  ];
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0,  1.0, 0.0, 1.0,  2.0, 0.0, 0.5];
        let mut mesh = Mesh::new(indices, positions);

        while mesh.no_faces() > 1 {
            for halfedge_id in mesh.halfedge_iter() {
                if mesh.is_edge_on_boundary(halfedge_id)
                {
                    mesh.collapse_edge(halfedge_id);
                    break;
                }
            }
        }
        assert_eq!(mesh.no_vertices(), 3);
        assert_eq!(mesh.no_halfedges(), 6);
        assert_eq!(mesh.no_faces(), 1);
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_remove_face_when_unconnected()
    {
        let positions: Vec<f64> = vec![1.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, -1.0,
                                       1.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, -1.0];
        let mut mesh = Mesh::new((0..6).collect(), positions);

        let faces: Vec<FaceID> = mesh.face_iter().into_iter().collect();

        mesh.remove_face(faces[0]);

        assert_eq!(3, mesh.no_vertices());
        assert_eq!(6, mesh.no_halfedges());
        assert_eq!(1, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_remove_face_when_connected()
    {
        let mut mesh = MeshBuilder::new().square().build().unwrap();

        let face_id = mesh.face_iter().next().unwrap();

        mesh.remove_face(face_id);

        assert_eq!(3, mesh.no_vertices());
        assert_eq!(6, mesh.no_halfedges());
        assert_eq!(1, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_remove_face_when_three_connected_faces()
    {
        let mut mesh = MeshBuilder::new().subdivided_triangle().build().unwrap();

        let face_id = mesh.face_iter().next().unwrap();

        mesh.remove_face(face_id);

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
        mesh.is_valid().unwrap();
    }
    
    #[test]
    fn test_split_vertex() {
        let mut mesh = MeshBuilder::new().icosahedron().build().unwrap();
        let no_faces = mesh.no_faces();
        let no_halfedges = mesh.no_halfedges();
        
        let vert = mesh.vertex_iter().next().unwrap();
        let mut walker = mesh.walker_from_vertex(vert);
        let start = walker.halfedge_id().unwrap();
        let end = walker.as_next().as_twin().as_next().as_twin().halfedge_id().unwrap();
        
        mesh.split_vertex(start, end);
        
        assert_eq!(13, mesh.no_vertices());
        assert_eq!(no_faces, mesh.no_faces());
        assert_eq!(no_halfedges+4, mesh.no_halfedges());
        mesh.is_valid().unwrap();
    }
    
    use std::iter::FromIterator;
    
    #[test]
    fn test_bevel_curve() {
        let mut mesh = MeshBuilder::new().icosahedron().build().unwrap();
        mesh.bevel_curve(&Vec::from_iter([0,1,4].iter().cloned().map(VertexID::new)), 0.1);
        mesh.is_valid();
        
        mesh = MeshBuilder::new().icosahedron().build().unwrap();
        mesh.bevel_curve(&Vec::from_iter([0,4,5,3,7,6].iter().cloned().map(VertexID::new)), 0.2);
        mesh.is_valid();
    }
}
