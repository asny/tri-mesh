
use dust::*;
use dust::objects::*;
use dust::window::{event::*, Window};

fn main() {
    let mut window = Window::new_default("Morph tool").unwrap();
    let (width, height) = window.size();
    let gl = window.gl();

    // Renderer
    let renderer = DeferredPipeline::new(&gl, width, height, true).unwrap();

    // Camera
    let mut camera = camera::PerspectiveCamera::new(dust::vec3(5.0, 3.0, 5.0), dust::vec3(0.0, 1.0, 0.0),
                                                    dust::vec3(0.0, 1.0, 0.0),degrees(45.0), width as f32 / height as f32, 0.1, 1000.0);

    // Objects
    let color = vec3(1.0, 0.0, 0.0);
    //let source = include_str!("bunny.obj").to_string();

    println!("Loading model");
    let mut meshes = geo_proc::loader::load_obj("examples/bunny.obj").unwrap();
    let mut mesh = meshes.drain(..).next().unwrap();
    println!("Model loaded");
    mesh.scale(20.0);
    mesh.translate(vec3(0.0, 1.0, 0.0));

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

    let scene_radius = 30.0;
    let mut light1 = light::SpotLight::new(vec3(scene_radius, scene_radius, scene_radius), vec3(-1.0, -1.0, -1.0));
    light1.enable_shadows(&gl, scene_radius * 2.0).unwrap();
    light1.base.intensity = 0.75;

    let mut light2 = light::SpotLight::new(vec3(-scene_radius, scene_radius, scene_radius), vec3(1.0, -1.0, -1.0));
    light2.enable_shadows(&gl, scene_radius * 2.0).unwrap();
    light2.base.intensity = 0.75;

    let mut light3 = light::SpotLight::new(vec3(-scene_radius, scene_radius, -scene_radius), vec3(1.0, -1.0, 1.0));
    light3.enable_shadows(&gl, scene_radius * 2.0).unwrap();
    light3.base.intensity = 0.75;

    let mut light4 = light::SpotLight::new(vec3(scene_radius, scene_radius, -scene_radius), vec3(-1.0, -1.0, 1.0));
    light4.enable_shadows(&gl, scene_radius * 2.0).unwrap();
    light4.base.intensity = 0.75;

    let mut camera_handler = camerahandler::CameraHandler::new(camerahandler::CameraState::SPHERICAL);

    let mut time = 0.0;
    // main loop
    window.render_loop(move |events, elapsed_time|
    {
        for event in events {
            handle_events(event, &mut camera_handler, &mut camera);
        }

        // Update scene
        time += elapsed_time * 0.001;
        mesh.translate(vec3(time.sin() as f32, 0.0, 0.0));
        model.update_attributes(&att!["position" => (mesh.positions_buffer(), 3), "normal" => (mesh.normals_buffer(), 3)]).unwrap();

        // Draw
        let render_scene = |camera: &Camera| {
            let model_matrix = Mat4::identity();
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
        plane.render(&Mat4::from_scale(100.0), &camera);

        // Light pass
        renderer.light_pass_begin(&camera).unwrap();
        renderer.shine_ambient_light(&ambient_light).unwrap();
        renderer.shine_spot_light(&light1).unwrap();
        renderer.shine_spot_light(&light2).unwrap();
        renderer.shine_spot_light(&light3).unwrap();
        renderer.shine_spot_light(&light4).unwrap();

        // Blend with the result of the mirror pass
        state::blend(&gl,state::BlendType::SRC_ALPHA__ONE_MINUS_SRC_ALPHA);
        state::depth_write(&gl,false);
        state::depth_test(&gl, state::DepthTestType::NONE);
        state::cull(&gl,state::CullType::BACK);

        renderer.copy_to_screen().unwrap();
    }).unwrap();
}

pub fn handle_events(event: &Event, camera_handler: &mut dust::camerahandler::CameraHandler, camera: &mut Camera)
{
    match event {
        Event::Key {state, kind} => {
            if kind == "Tab" && *state == State::Pressed
            {
                camera_handler.next_state();
            }
        },
        Event::MouseClick {state, button} => {
            if *button == MouseButton::Left
            {
                if *state == State::Pressed { camera_handler.start_rotation(); }
                else { camera_handler.end_rotation() }
            }
        },
        Event::MouseMotion {delta} => {
            camera_handler.rotate(camera, delta.0 as f32, delta.1 as f32);
        },
        Event::MouseWheel {delta} => {
            camera_handler.zoom(camera, *delta as f32);
        }
    }
}
