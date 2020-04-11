
use tri_mesh::prelude::*;

#[derive(Debug)]
pub struct StitchOperation {
    mesh: Mesh
}

impl StitchOperation {
    pub fn new(scene_radius: f64) -> Self {
        let mut mesh = MeshBuilder::new().with_obj(include_str!("../assets/blob.obj").to_string()).build().unwrap();
        let (min, max) = mesh.extreme_coordinates();
        mesh.translate(-0.5 * (max + min)); // Translate such that the mesh center is in origo.
        let size = max - min;
        mesh.scale(0.5 * scene_radius / size.x.max(size.y).max(size.z)); // Scale the mesh such that the size of the biggest side of the bounding box is half a scene radius
        Self {mesh}
    }

    pub fn apply(&mut self, mesh: &Mesh, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<Mesh>
    {
        if let Some(Intersection::Point {point, ..}) = mesh.ray_intersection(ray_start_point, ray_direction) {
            // Translate
            self.mesh.translate(point - self.mesh.axis_aligned_bounding_box_center());

            // Split at intersection
            let (meshes1, meshes2) = mesh.clone().split_at_intersection(&mut self.mesh);

            // Merge sub meshes
            //let mut result_meshes = Vec::new();
            for mesh1 in meshes1.iter() {
                for mesh2 in meshes2.iter() {
                    let mut result = mesh1.clone();
                    if result.merge_with(mesh2).is_ok()
                    {
                        return Some(result);
                        //result_meshes.push(result);
                    }
                }
            }
        }
        None
    }
}

/*Event::MouseClick {state, button, position} => {
                    if *button == MouseButton::Left
                    {
                        if *state == State::Pressed
                        {

                            if let Some(ref result) = results {
                                if let Some(Intersection::Point {..}) = result[chosen].0.ray_intersection(&ray_start_point, &ray_direction) {
                                    chosen = (chosen + 1) % result.len();
                                }
                                else {
                                    camera_handler.start_rotation();
                                }
                            }
                            else {
                                if let Some(mut meshes) = on_click(&mut mesh, &mut other_mesh, &ray_start_point, &ray_direction) {
                                    let mut result = Vec::new();
                                    for mesh in meshes.drain(..) {
                                        let positions: Vec<f32> = mesh.positions_buffer().iter().map(|v| *v as f32).collect();
                                        let normals: Vec<f32> = mesh.normals_buffer().iter().map(|v| *v as f32).collect();

                                        let mut wireframe_model = Wireframe::new(&gl, &mesh.indices_buffer(), &positions, 0.02);
                                        wireframe_model.set_parameters(0.8, 0.2, 5.0);
                                        wireframe_model.set_color(&vec3(0.9, 0.2, 0.2));

                                        let mut model = ShadedMesh::new(&gl, &mesh.indices_buffer(), &att!["position" => (positions, 3), "normal" => (normals, 3)]).unwrap();
                                        model.color = vec3(0.8, 0.8, 0.8);

                                        result.push((mesh, model, wireframe_model));
                                    }
                                    results = Some(result);

                                }
                                else {
                                    camera_handler.start_rotation();
                                }
                            }
                        }
                        else {
                            camera_handler.end_rotation()
                        }
                    }
                    else if *button == MouseButton::Right && *state == State::Pressed
                    {
                        results = None;
                        chosen = 0;
                    }
                },*/