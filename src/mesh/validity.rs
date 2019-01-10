use crate::mesh::Mesh;

#[derive(Debug)]
pub enum Error {
    IsNotValid {message: String}
}

impl Mesh
{
    pub fn test_is_valid(&self) -> Result<(), Error>
    {
        for vertex_id in self.vertex_iter() {
            if let Some(halfedge_id) = self.walker_from_vertex(&vertex_id).halfedge_id()
            {
                if !self.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                    return Err(Error::IsNotValid {message: format!("Vertex {} points to an invalid halfedge {}", vertex_id, halfedge_id)});
                }
                if self.walker_from_vertex(&vertex_id).as_twin().vertex_id().unwrap() != vertex_id
                {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} pointed to by vertex {} does not start in that vertex, but instead in {}", self.walker_from_vertex(&vertex_id).halfedge_id().unwrap(), vertex_id, self.walker_from_vertex(&vertex_id).as_twin().vertex_id().unwrap())});
                }
            }
            else {
                return Err(Error::IsNotValid {message: format!("Vertex {} does not point to a halfedge", vertex_id)});
            }
        }
        for halfedge_id in self.halfedge_iter() {
            let walker = self.walker_from_halfedge(&halfedge_id);

            if let Some(twin_id) = walker.twin_id()
            {
                if !self.halfedge_iter().any(|he_id| he_id == twin_id) {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid twin halfedge {}", halfedge_id, twin_id)});
                }
                if self.walker_from_halfedge(&twin_id).twin_id().unwrap() != halfedge_id
                {
                    return Err(Error::IsNotValid {message: format!("Halfedge twin pointed to by halfedge {} does not point back to halfedge", halfedge_id)});
                }
                if self.walker_from_halfedge(&twin_id).vertex_id() == walker.vertex_id()
                {
                    return Err(Error::IsNotValid {message: format!("Invalid orientation: The halfedge {} and its twin halfedge {} points to the same vertex {}", halfedge_id, twin_id, walker.vertex_id().unwrap())});
                }
            }
            else {
                return Err(Error::IsNotValid {message: format!("Halfedge {} does not point to a twin halfedge", halfedge_id)});
            }

            if let Some(vertex_id) = walker.vertex_id()
            {
                if !self.vertex_iter().any(|vid| vid == vertex_id) {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid vertex {}", halfedge_id, vertex_id)});
                }
            }
            else {
                return Err(Error::IsNotValid {message: format!("Halfedge {} does not point to a vertex", halfedge_id)});
            }

            if let Some(face_id) = walker.face_id()
            {
                if !self.face_iter().any(|fid| fid == face_id) {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid face {}", halfedge_id, face_id)});
                }
                if walker.next_id().is_none() {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to a face but not a next halfedge", halfedge_id)});
                }
            }

            if let Some(next_id) = walker.next_id()
            {
                if !self.halfedge_iter().any(|he_id| he_id == next_id) {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid next halfedge {}", halfedge_id, next_id)});
                }
                if walker.face_id().is_none() {
                    return Err(Error::IsNotValid {message: format!("Halfedge {} points to a next halfedge but not a face", halfedge_id)});
                }
                if self.walker_from_halfedge(&next_id).previous_id().unwrap() != halfedge_id
                {
                    return Err(Error::IsNotValid {message: format!("Halfedge next pointed to by halfedge {} does not point back to halfedge", halfedge_id)});
                }
            }

            if self.edge_length(&halfedge_id) < 0.00001
            {
                return Err(Error::IsNotValid {message: format!("Length of edge {} is too small ({})", halfedge_id, self.edge_length(&halfedge_id))})
            }
        }
        for face_id in self.face_iter() {
            if let Some(halfedge_id) = self.walker_from_face(&face_id).halfedge_id()
            {
                if !self.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                    return Err(Error::IsNotValid {message: format!("Face {} points to an invalid halfedge {}", face_id, halfedge_id)});
                }
                if self.walker_from_face(&face_id).face_id().unwrap() != face_id
                {
                    return Err(Error::IsNotValid {message: format!("Halfedge pointed to by face {} does not point to back to face", face_id)});
                }
            }
            else {
                return Err(Error::IsNotValid {message: format!("Face {} does not point to a halfedge", face_id)});
            }

            if self.face_area(&face_id) < 0.00001
            {
                return Err(Error::IsNotValid {message: format!("Area of face {} is too small ({})", face_id, self.face_area(&face_id))})
            }
        }

        for vertex_id1 in self.vertex_iter()
        {
            for vertex_id2 in self.vertex_iter()
            {
                if self.connecting_edge(&vertex_id1, &vertex_id2).is_some() != self.connecting_edge(&vertex_id2, &vertex_id1).is_some()
                {
                    return Err(Error::IsNotValid {message: format!("Vertex {} and Vertex {} is connected one way, but not the other way", vertex_id1, vertex_id2)});
                }
                let mut found = false;
                for halfedge in self.vertex_halfedge_iter(&vertex_id1) {
                    if halfedge.vertex_id().unwrap() == vertex_id2 {
                        if found {
                            return Err(Error::IsNotValid {message: format!("Vertex {} and Vertex {} is connected by multiple edges", vertex_id1, vertex_id2)})
                        }
                        found = true;
                    }
                }
            }
        }
        Ok(())
    }
}
