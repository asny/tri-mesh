
use three_d::*;

mod morph;

fn main()
{
    let scene_radius = 10.0;
    let scene_center = vec3(0.0, 5.0, 0.0);

    // Window
    let mut window = Window::new_default("Mesh tool").unwrap();
    let (width, height) = window.framebuffer_size();
    let window_size = window.size();
    let gl = window.gl();

    // Renderer
    let mut renderer = DeferredPipeline::new(&gl).unwrap();
    let mut camera = Camera::new_perspective(&gl, scene_center + scene_radius * vec3(1.0, 1.0, 1.0).normalize(), scene_center, vec3(0.0, 1.0, 0.0),
                                                degrees(45.0), width as f32 / height as f32, 0.1, 1000.0);

    // Lights
    let ambient_light = AmbientLight::new(&gl, 0.4, &vec3(1.0, 1.0, 1.0)).unwrap();
    let mut dir = vec3(-1.0, -1.0, -1.0).normalize();
    let mut spot_light0 = SpotLight::new(&gl, 0.6, &vec3(1.0, 1.0, 1.0), &(scene_center - 2.0f32 * scene_radius * dir),
                                   &dir, 25.0, 0.1, 0.001, 0.0001).unwrap();
    dir = vec3(1.0, -1.0, -1.0).normalize();
    let mut spot_light1 = SpotLight::new(&gl, 0.6, &vec3(1.0, 1.0, 1.0), &(scene_center - 2.0f32 * scene_radius * dir),
                                   &dir, 25.0, 0.1, 0.001, 0.0001).unwrap();
    dir = vec3(1.0, -1.0, 1.0).normalize();
    let mut spot_light2 = SpotLight::new(&gl, 0.6, &vec3(1.0, 1.0, 1.0), &(scene_center - 2.0f32 * scene_radius * dir),
                                   &dir, 25.0, 0.1, 0.001, 0.0001).unwrap();
    dir = vec3(-1.0, -1.0, 1.0).normalize();
    let mut spot_light3 = SpotLight::new(&gl, 0.6, &vec3(1.0, 1.0, 1.0), &(scene_center - 2.0f32 * scene_radius * dir),
                                   &dir, 25.0, 0.1, 0.001, 0.0001).unwrap();

    // Plane
    let mut plane_mesh = tri_mesh::MeshBuilder::new().plane().build().unwrap();
    plane_mesh.scale(100.0);
    let mut plane = Mesh::new(&gl, &plane_mesh.indices_buffer(), &plane_mesh.positions_buffer_f32(), &plane_mesh.normals_buffer_f32()).unwrap();
    plane.color = vec3(0.8, 0.8, 0.8);
    plane.diffuse_intensity = 0.2;
    plane.specular_intensity = 0.4;
    plane.specular_power = 20.0;

    // Mesh
    let mut mesh = tri_mesh::MeshBuilder::new().with_3d(include_bytes!("assets/suzanne.3d")).unwrap().build().unwrap();
    let (min, max) = mesh.extreme_coordinates();
    mesh.translate(-0.5 * (max + min)); // Translate such that the mesh center is in origo.
    let size = max - min;
    mesh.scale(0.5 * scene_radius as f64 / size.x.max(size.y).max(size.z)); // Scale the mesh such that the size of the biggest side of the bounding box is half a scene radius
    mesh.translate(tri_mesh::prelude::vec3(scene_center.x as f64, scene_center.y as f64, scene_center.z as f64)); // Translate the mesh to the scene center

    let mut drawable_mesh = DrawableMesh::new(&gl,&mesh);

    let mut morph_operation = None;

    // main loop
    let mut rotating = false;
    window.render_loop(move |frame_input|
    {
        camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

        for event in frame_input.events.iter() {
            match event {
                Event::MouseClick {state, button, position} => {
                    if *button == MouseButton::Left
                    {
                        if *state == State::Pressed
                        {
                            let (x, y) = (position.0 / window_size.0 as f64, position.1 / window_size.1 as f64);
                            let p = camera.position();
                            let dir = camera.view_direction_at((x, y));
                            morph_operation = morph::MorphOperation::new(&mesh,&tri_mesh::prelude::vec3(p.x as f64, p.y as f64, p.z as f64), &tri_mesh::prelude::vec3(dir.x as f64, dir.y as f64, dir.z as f64));
                            if morph_operation.is_none() {
                                rotating = true;
                            }
                        }
                        else {
                            morph_operation = None;
                            rotating = false;
                        }
                    }
                },
                Event::MouseWheel {delta} => {
                    camera.zoom(*delta as f32);
                },
                Event::MouseMotion {delta} => {
                    if rotating {
                        camera.rotate(delta.0 as f32, delta.1 as f32);
                    }
                    if let Some(ref operation) = morph_operation
                    {
                        operation.apply(&mut mesh, 0.001 * delta.1);
                        drawable_mesh.update(&mesh);
                    }
                },
                Event::Key { ref state, ref kind } => {
                    if kind == "Key1" && *state == State::Pressed
                    {
                        println!("Begin morph");
                    }
                }
            }
        }

        let render_scene = |camera: &Camera| {
            state::cull(&gl, state::CullType::Back);
            drawable_mesh.render_shadows(camera);
        };
        spot_light0.generate_shadow_map(50.0, 512, &render_scene);
        spot_light1.generate_shadow_map(50.0, 512, &render_scene);
        spot_light2.generate_shadow_map(50.0, 512, &render_scene);
        spot_light3.generate_shadow_map(50.0, 512, &render_scene);

        // Geometry pass
        renderer.geometry_pass(width, height, &|| {
            state::cull(&gl, state::CullType::Back);
            plane.render(&Mat4::identity(), &camera);
            drawable_mesh.render(&camera);
        }).unwrap();

        // Light pass
        Screen::write(&gl, 0, 0, width, height, Some(&vec4(0.5, 0.5, 0.5, 1.0)), None, &|| {
            renderer.light_pass(&camera, Some(&ambient_light), &[], &[&spot_light0, &spot_light1, &spot_light2, &spot_light3], &[]).unwrap();
        }).unwrap();
    }).unwrap();
}

pub struct DrawableMesh {
    wireframe: Edges,
    model: Mesh
}

impl DrawableMesh {
    pub fn new(gl: &Gl, mesh: &tri_mesh::mesh::Mesh) -> Self
    {
        let positions: Vec<f32> = mesh.positions_buffer().iter().map(|v| *v as f32).collect();
        let normals: Vec<f32> = mesh.normals_buffer().iter().map(|v| *v as f32).collect();

        let mut wireframe = Edges::new(gl, &mesh.indices_buffer(), &positions, 0.01);
        wireframe.diffuse_intensity = 0.8;
        wireframe.specular_intensity = 0.2;
        wireframe.specular_power = 5.0;
        wireframe.color = vec3(0.9, 0.2, 0.2);

        let mut model = Mesh::new(gl, &mesh.indices_buffer(), &positions, &normals).unwrap();
        model.color = vec3(0.8, 0.8, 0.8);
        model.diffuse_intensity = 0.2;
        model.specular_intensity = 0.4;
        model.specular_power = 20.0;

        Self { wireframe, model }
    }

    pub fn update(&mut self, mesh: &tri_mesh::mesh::Mesh)
    {
        let positions: Vec<f32> = mesh.positions_buffer().iter().map(|v| *v as f32).collect();
        let normals: Vec<f32> = mesh.normals_buffer().iter().map(|v| *v as f32).collect();
        self.wireframe.update_positions(&positions);
        self.model.update_positions(&positions).unwrap();
        self.model.update_normals(&normals).unwrap();
    }

    pub fn render_shadows(&self, camera: &Camera) {
        self.model.render(&Mat4::identity(), camera);
    }

    pub fn render(&self, camera: &Camera) {
        self.model.render(&Mat4::identity(), camera);
        self.wireframe.render(&Mat4::identity(), camera);
    }
}