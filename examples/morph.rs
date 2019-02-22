
use tri_mesh::prelude::*;
use tri_mesh::prelude::Vec3 as Vec3;
use tri_mesh::prelude::vec3 as vec3;
use tri_mesh::prelude::vec4 as vec4;
use std::collections::HashMap;

fn construct_mesh(scene_center: &Vec3, scene_radius: f32) -> Mesh
{
    let mut mesh = MeshBuilder::new().with_obj(include_str!("assets/bunny.obj").to_string()).build().unwrap();
    let (min, max) = mesh.extreme_coordinates();
    let center = 0.5 * (max + min);
    mesh.translate(-center);
    let size = max - min;
    let max_dim = size.x.max(size.y).max(size.z);
    mesh.scale(0.5 * scene_radius / max_dim);
    mesh.translate(*scene_center);
    mesh
}

fn pick(mesh: &Mesh, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<(VertexID, Vec3)>
{
    if let Some(Intersection::Point {primitive, point}) = mesh.ray_intersection(ray_start_point, ray_direction) {
        let start_vertex_id = match primitive {
            Primitive::Face(face_id) => {
                mesh.walker_from_face(face_id).vertex_id().unwrap()
            },
            Primitive::Edge((vertex_id, _)) => {
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

fn compute_weights(mesh: &Mesh, start_vertex_id: VertexID, start_point: &Vec3) -> HashMap<VertexID, Vec3>
{
    static SQR_MAX_DISTANCE: f32 = 1.0;
    let mut weights = HashMap::new();
    visit_vertices(mesh, start_vertex_id, &mut |mesh, vertex_id| {
        let d = start_point.distance2(*mesh.vertex_position(vertex_id));

        if d < SQR_MAX_DISTANCE
        {
            weights.insert(vertex_id, weight(d, SQR_MAX_DISTANCE) * mesh.vertex_normal(vertex_id));
            true
        }
        else {false}
    });
    weights
}

fn weight(distance: f32, max_distance: f32) -> f32
{
    let x = distance / max_distance;
    1.0 - x*x*(3.0 - 2.0 * x)
}

fn visit_vertices(mesh: &Mesh, start_vertex_id: VertexID, callback: &mut FnMut(&Mesh, VertexID) -> bool)
{
    let mut component = std::collections::HashSet::new();
    component.insert(start_vertex_id);
    let mut to_be_tested = vec![start_vertex_id];
    while let Some(test_id) = to_be_tested.pop()
    {
        if callback(mesh, test_id)
        {
            for halfedge_id in mesh.vertex_halfedge_iter(test_id)
            {
                let vertex_id = mesh.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                if !component.contains(&vertex_id) {
                    to_be_tested.push(vertex_id);
                    component.insert(vertex_id);
                }
            }
        }
    }
}

fn morph(mesh: &mut Mesh, weights: &HashMap<VertexID, Vec3>, factor: f32)
{
    for (vertex_id, weight) in weights.iter() {
        mesh.move_vertex_by(*vertex_id,weight * factor);
    }
}

///
/// Above: Everything related to tri-mesh
/// Below: Visualisation of the mesh, event handling and so on
///
use dust::*;
use dust::objects::*;
use dust::window::{event::*, Window};

fn main()
{
    let scene_radius = 10.0;
    let scene_center = vec3(0.0, 5.0, 0.0);
    let mut mesh = construct_mesh(&scene_center, scene_radius);

    let mut window = Window::new_default("Morph tool").unwrap();
    let (framebuffer_width, framebuffer_height) = window.framebuffer_size();
    let window_size = window.size();
    let gl = window.gl();

    // Renderer
    let renderer = DeferredPipeline::new(&gl, framebuffer_width, framebuffer_height, true, vec4(0.8, 0.8, 0.8, 1.0)).unwrap();

    // Camera
    let mut camera = camera::PerspectiveCamera::new(scene_center + scene_radius * vec3(1.0, 1.0, 1.0).normalize(), scene_center,
                                                    vec3(0.0, 1.0, 0.0),degrees(45.0), framebuffer_width as f32 / framebuffer_height as f32, 0.1, 1000.0);

    // Objects
    let mut model = ShadedMesh::new(&gl, &mesh.indices_buffer(), &att!["position" => (mesh.positions_buffer(), 3), "normal" => (mesh.normals_buffer(), 3)]).unwrap();
    model.color = vec3(0.8, 0.8, 0.8);

    let mut wireframe_model = Wireframe::new(&gl, &mesh.indices_buffer(), &mesh.positions_buffer(), 0.02);
    wireframe_model.set_parameters(0.8, 0.2, 5.0);
    wireframe_model.set_color(&vec3(0.9, 0.2, 0.2));

    let plane_positions: Vec<f32> = vec![
        -1.0, 0.0, -1.0,
        1.0, 0.0, -1.0,
        1.0, 0.0, 1.0,
        -1.0, 0.0, 1.0
    ];
    let plane_normals: Vec<f32> = vec![
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0
    ];
    let plane_indices: Vec<u32> = vec![
        0, 2, 1,
        0, 3, 2,
    ];
    let mut plane = ShadedMesh::new(&gl, &plane_indices, &att!["position" => (plane_positions, 3), "normal" => (plane_normals, 3)]).unwrap();
    plane.diffuse_intensity = 0.2;
    plane.specular_intensity = 0.4;
    plane.specular_power = 20.0;

    let mut ambient_light = light::AmbientLight::new();
    ambient_light.base.intensity = 0.4;

    let mut dir = vec3(-1.0, -1.0, -1.0).normalize();
    let mut light1 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light1.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light1.base.intensity = 0.75;

    dir = vec3(-1.0, -1.0, 1.0).normalize();
    let mut light2 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light2.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light2.base.intensity = 0.75;

    dir = vec3(1.0, -1.0, 1.0).normalize();
    let mut light3 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light3.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light3.base.intensity = 0.75;

    dir = vec3(1.0, -1.0, -1.0).normalize();
    let mut light4 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light4.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light4.base.intensity = 0.75;

    let mut camera_handler = camerahandler::CameraHandler::new(camerahandler::CameraState::SPHERICAL);

    let mut weights: Option<HashMap<VertexID, Vec3>> = None;
    // main loop
    window.render_loop(move |events, _elapsed_time|
    {
        for event in events {
            match event {
                Event::Key {state, kind} => {
                    if kind == "Tab" && *state == State::Pressed
                    {
                        camera_handler.next_state();
                    }
                },
                Event::MouseClick {state, button, position} => {
                    if *button == MouseButton::Left
                    {
                        if *state == State::Pressed
                        {
                            let (x, y) = (position.0 / window_size.0 as f64, position.1 / window_size.1 as f64);
                            let p = camera.position();
                            let dir = camera.view_direction_at((x, y));
                            if let Some((vertex_id, point)) = pick(&mesh,&p, &dir) {
                                weights = Some(compute_weights(&mesh, vertex_id, &point));
                            }
                            else {
                                camera_handler.start_rotation();
                            }
                        }
                        else {
                            weights = None;
                            camera_handler.end_rotation()
                        }
                    }
                },
                Event::MouseWheel {delta} => {
                    camera_handler.zoom(&mut camera, *delta as f32);
                },
                Event::MouseMotion {delta} => {
                    camera_handler.rotate(&mut camera, delta.0 as f32, delta.1 as f32);
                    if let Some(ref w) = weights
                    {
                        morph(&mut mesh, w, 0.001 * delta.1 as f32);
                        model.update_attributes(&att!["position" => (mesh.positions_buffer(), 3), "normal" => (mesh.normals_buffer(), 3)]).unwrap();
                        wireframe_model.update_positions(&mesh.positions_buffer());
                    }
                }
            }
        }

        // Draw
        let render_scene = |camera: &Camera| {
            let model_matrix = dust::Mat4::identity();
            model.render(&model_matrix, camera);
            wireframe_model.render(camera);
        };

        // Shadow pass
        light1.shadow_cast_begin().unwrap();
        render_scene(light1.shadow_camera().unwrap());

        light2.shadow_cast_begin().unwrap();
        render_scene(light2.shadow_camera().unwrap());

        light3.shadow_cast_begin().unwrap();
        render_scene(light3.shadow_camera().unwrap());

        light4.shadow_cast_begin().unwrap();
        render_scene(light4.shadow_camera().unwrap());

        // Geometry pass
        renderer.geometry_pass_begin().unwrap();
        render_scene(&camera);
        plane.render(&dust::Mat4::from_scale(100.0), &camera);

        // Light pass
        renderer.light_pass_begin(&camera).unwrap();
        renderer.shine_ambient_light(&ambient_light).unwrap();
        renderer.shine_spot_light(&light1).unwrap();
        renderer.shine_spot_light(&light2).unwrap();
        renderer.shine_spot_light(&light3).unwrap();
        renderer.shine_spot_light(&light4).unwrap();

        renderer.copy_to_screen().unwrap();
    }).unwrap();
}