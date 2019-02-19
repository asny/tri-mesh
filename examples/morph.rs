
use dust::*;
use dust::objects::*;
use dust::window::{event::*, Window};
use geo_proc::prelude::*;

fn main() {
    let mut window = Window::new_default("Morph tool").unwrap();
    let (framebuffer_width, framebuffer_height) = window.framebuffer_size();
    let window_size = window.size();
    let gl = window.gl();

    let scene_radius = 10.0;
    let scene_center = dust::vec3(0.0, 5.0, 0.0);

    // Renderer
    let renderer = DeferredPipeline::new(&gl, framebuffer_width, framebuffer_height, true).unwrap();

    // Camera
    let mut camera = camera::PerspectiveCamera::new(scene_center + scene_radius * dust::vec3(1.0, 1.0, 1.0).normalize(), scene_center,
                                                    dust::vec3(0.0, 1.0, 0.0),degrees(45.0), framebuffer_width as f32 / framebuffer_height as f32, 0.1, 1000.0);

    // Objects
    let color = dust::vec3(1.0, 0.0, 0.0);
    //let source = include_str!("bunny.obj").to_string();

    println!("Loading model");
    let mut meshes = geo_proc::loader::load_obj("examples/bunny.obj").unwrap();
    let mut mesh = meshes.drain(..).next().unwrap();
    println!("Model loaded");
    let (min, max) = mesh.extreme_coordinates();
    let center = 0.5 * (max + min);
    mesh.translate(-center);
    let size = max - min;
    let max_dim = size.x.max(size.y).max(size.z);
    mesh.scale(0.5 * scene_radius / max_dim);
    mesh.translate(scene_center);

    let mut model = ShadedMesh::new(&gl, &mesh.indices_buffer(), &att!["position" => (mesh.positions_buffer(), 3), "normal" => (mesh.normals_buffer(), 3)]).unwrap();
    model.color = color;

    let mut wireframe_model = Wireframe::new(&gl, &mesh.indices_buffer(), &mesh.positions_buffer(), 0.02);
    wireframe_model.set_parameters(0.8, 0.2, 5.0);
    wireframe_model.set_color(&color);

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
    let mut plane = crate::objects::ShadedMesh::new(&gl, &plane_indices, &att!["position" => (plane_positions, 3), "normal" => (plane_normals, 3)]).unwrap();
    plane.diffuse_intensity = 0.2;
    plane.specular_intensity = 0.4;
    plane.specular_power = 20.0;

    let mut ambient_light = light::AmbientLight::new();
    ambient_light.base.intensity = 0.4;

    let mut dir = dust::vec3(-1.0, -1.0, -1.0).normalize();
    let mut light1 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light1.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light1.base.intensity = 0.75;

    dir = dust::vec3(-1.0, -1.0, 1.0).normalize();
    let mut light2 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light2.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light2.base.intensity = 0.75;

    dir = dust::vec3(1.0, -1.0, 1.0).normalize();
    let mut light3 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light3.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light3.base.intensity = 0.75;

    dir = dust::vec3(1.0, -1.0, -1.0).normalize();
    let mut light4 = light::SpotLight::new(scene_center - 2.0 * scene_radius * dir, dir);
    light4.enable_shadows(&gl, scene_radius * 4.0).unwrap();
    light4.base.intensity = 0.75;

    let mut camera_handler = camerahandler::CameraHandler::new(camerahandler::CameraState::SPHERICAL);

    // main loop
    window.render_loop(move |events, _elapsed_time|
    {
        for event in events {
            handle_events(event, &mut camera_handler, &mut camera, &mut mesh, window_size);
        }

        // Update scene
        model.update_attributes(&att!["position" => (mesh.positions_buffer(), 3), "normal" => (mesh.normals_buffer(), 3)]).unwrap();

        // Draw
        let render_scene = |camera: &Camera| {
            let model_matrix = dust::Mat4::identity();
            model.render(&model_matrix, camera);
            //wireframe_model.render(camera);
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

pub fn handle_events(event: &Event, camera_handler: &mut dust::camerahandler::CameraHandler, camera: &mut Camera, mesh: &mut Mesh, window_size: (usize, usize))
{
    static mut CURRENT: Option<(VertexID, dust::Vec3)> = None;

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
                if *state == State::Pressed { camera_handler.start_rotation(); }
                else { camera_handler.end_rotation() }
            }
            else if *button == MouseButton::Right {
                if *state == State::Pressed
                {
                    let (x, y) = (position.0 / window_size.0 as f64, position.1 / window_size.1 as f64);
                    let p = camera.position();
                    println!("{:?}", (position.0, position.1));
                    let dir = get_view_direction_at(camera, (x, y));
                    if let Some(intersection) = pick(mesh, &p, &dir) {
                        println!("{:?}", intersection);
                        unsafe {CURRENT = Some(intersection)}
                    }
                }
                else {
                    unsafe {CURRENT = None};
                }
            }
        },
        Event::MouseMotion {delta} => {
            camera_handler.rotate(camera, delta.0 as f32, delta.1 as f32);

            unsafe {
                if let Some((vertex_id, point)) = CURRENT
                {
                    mesh.move_vertex_by(vertex_id,0.01 * delta.1 as f32 * mesh.vertex_normal(vertex_id));
                }
            }
        },
        Event::MouseWheel {delta} => {
            camera_handler.zoom(camera, *delta as f32);
        }
    }
}

fn pick(mesh: &Mesh, point: &geo_proc::prelude::Vec3, direction: &geo_proc::prelude::Vec3) -> Option<(VertexID, dust::Vec3)>
{
    use geo_proc::collision::*;
    let mut current: Option<FaceLinePieceIntersection> = None;
    for face_id in mesh.face_iter() {
        if let Some(intersection) = find_face_line_piece_intersection(mesh, face_id, point, &(point + direction * 100.0))
        {
            if let Some(ref mut c) = current {
                if c.point.distance2(*point) > intersection.point.distance2(*point) {
                    *c = intersection;
                }
            }
            else {
                current = Some(intersection);
            }
        }
    }
    if let Some(intersection) = current {
        match intersection.id {
            Primitive::Face(face_id) => {
                let vertex_id = mesh.walker_from_face(face_id).vertex_id().unwrap();
                return Some((vertex_id, intersection.point));
            },
            Primitive::Edge((vertex_id, _)) => {
                return Some((vertex_id, intersection.point));
            },
            Primitive::Vertex(vertex_id) => {
                return Some((vertex_id, intersection.point));
            }
        }
    }
    None
}

fn get_view_direction_at(camera: &Camera, screen_uv: (f64, f64)) -> dust::Vec3
{
    let screen_pos = dust::vec4(2. * screen_uv.0 as f32 - 1., 1. - 2. * screen_uv.1 as f32, 1., 1.);
    let mut ray_eye = camera.get_projection().invert().unwrap() * screen_pos;
    ray_eye = dust::vec4(ray_eye.x, ray_eye.y, -1.0, 0.0);
    let ray_world = camera.get_view().invert().unwrap() *  ray_eye;
    dust::vec3(ray_world.x, ray_world.y, ray_world.z).normalize()
}