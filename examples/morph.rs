
use std::collections::HashMap;
use tri_mesh::prelude::*;

pub struct MorphOperation {
    weights: HashMap<VertexID, Vec3>
}

impl MorphOperation {
    pub fn new(mesh: &Mesh, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<Self> {

        if let Some((vertex_id, point)) = pick(&mesh,&ray_start_point, &ray_direction) {
            Some(Self { weights: compute_weights(mesh, vertex_id, &point) })
        }
        else {None}
    }

    pub fn apply(&self, mesh: &mut Mesh, factor: f64)
    {
        for (vertex_id, weight) in self.weights.iter() {
            mesh.move_vertex_by(*vertex_id,weight * factor);
        }
    }

}

/// Loads the mesh and scale/translate it.
pub fn on_startup(scene_center: &tri_mesh::prelude::Vec3, scene_radius: f64) -> tri_mesh::mesh::Mesh
{
    use tri_mesh::prelude::*;
    let mut mesh = MeshBuilder::new().with_3d(include_bytes!("assets/suzanne.3d")).unwrap().build().unwrap();
    let (min, max) = mesh.extreme_coordinates();
    mesh.translate(-0.5 * (max + min)); // Translate such that the mesh center is in origo.
    let size = max - min;
    mesh.scale(0.5 * scene_radius / size.x.max(size.y).max(size.z)); // Scale the mesh such that the size of the biggest side of the bounding box is half a scene radius
    mesh.translate(*scene_center); // Translate the mesh to the scene center
    mesh
}

/// Picking used for determining whether a mouse click starts a morph operation. Returns a close vertex and the position of the click on the mesh surface.
fn pick(mesh: &Mesh, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<(VertexID, Vec3)>
{
    use tri_mesh::prelude::*;
    if let Some(Intersection::Point {primitive, point}) = mesh.ray_intersection(ray_start_point, ray_direction) {
        let start_vertex_id = match primitive {
            Primitive::Face(face_id) => {
                mesh.walker_from_face(face_id).vertex_id().unwrap()
            },
            Primitive::Edge(halfedge_id) => {
                let (vertex_id, ..) = mesh.edge_vertices(halfedge_id);
                vertex_id
            },
            Primitive::Vertex(vertex_id) => {
                vertex_id
            }
        };
        Some((start_vertex_id, point))
    }
    else {None}
}

/// Compute a directional weight for each vertex to be used for the morph operation.
fn compute_weights(mesh: &Mesh, start_vertex_id: VertexID, start_point: &Vec3) -> HashMap<VertexID, Vec3>
{
    use tri_mesh::prelude::*;
    static SQR_MAX_DISTANCE: f64 = 1.0;

    // Use the smoothstep function to get a smooth morphing
    let smoothstep_function = |sqr_distance| {
        let x = sqr_distance / SQR_MAX_DISTANCE;
        1.0 - x*x*(3.0 - 2.0 * x)
    };

    // Visit all the vertices close to the start vertex.
    let mut weights = HashMap::new();
    let mut to_be_tested = vec![start_vertex_id];
    while let Some(vertex_id) = to_be_tested.pop()
    {
        let sqr_distance = start_point.distance2(mesh.vertex_position(vertex_id));
        if sqr_distance < SQR_MAX_DISTANCE
        {
            // The weight is computed as the smoothstep function to the square euclidean distance
            // to the start point on the surface multiplied by the vertex normal.
            weights.insert(vertex_id, smoothstep_function(sqr_distance) * mesh.vertex_normal(vertex_id));

            // Add neighbouring vertices to be tested if they have not been visited yet
            for halfedge_id in mesh.vertex_halfedge_iter(vertex_id)
            {
                let neighbour_vertex_id = mesh.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                if !weights.contains_key(&neighbour_vertex_id) {
                    to_be_tested.push(neighbour_vertex_id);
                }
            }
        }
    }
    weights
}