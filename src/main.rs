use std::{
    thread,
    time::{Duration, SystemTime},
};

use glm::Vec3;
use render::Render;
use sdl2::{event::WindowEvent, keyboard::Keycode};

mod brush;
mod entity;
#[macro_use]
mod gl_wrappers;
mod render;
mod vector3;
mod vertex;
mod world;

use vertex::{TriangleMesh, Vertex};
use world::World;

#[derive(Default, Debug)]
struct ScreenSpaceMesh {
    triangles: Vec<vertex::TriangleMesh>,
}

impl ScreenSpaceMesh {
    fn add_tri(&mut self, tri: vertex::TriangleMesh) {
        self.triangles.push(tri);
    }
    fn clear(&mut self) {
        self.triangles.clear();
    }
}

struct Camera {
    pos: Vec3,
    /// XYZ Euler angles. (0,0,0) means upwards.
    /// X: Roll
    /// Y: Pitch
    /// Z: Yaw
    orientation: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            orientation: glm::vec3(-90.0, 0.0, 0.0),
            pos: glm::to_vec3(0.),
        }
    }
}

fn main() {
    let sdl_ctx = sdl2::init().unwrap();

    let video_ctx = sdl_ctx.video().unwrap();
    video_ctx.gl_load_library_default().unwrap();

    video_ctx
        .gl_attr()
        .set_context_flags()
        .forward_compatible()
        .debug()
        .set();
    video_ctx.gl_attr().set_context_major_version(4);
    video_ctx.gl_attr().set_context_minor_version(1);
    let window = video_ctx
        .window("SDL", 800, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let main_id = window.id();

    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    unsafe {
        gl::Viewport(0, 0, 800, 600);
    }

    let gl_ctx = window.gl_create_context().unwrap();
    let mut render_ctx = Render::init(&gl_ctx);

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut _world = World::new();
    let mut screen_world = ScreenSpaceMesh::default();
    let mut _camera = Camera::default();
    screen_world.add_tri(TriangleMesh(
        // Left
        Vertex {
            pos: glm::vec3(-0.5, -0.5, 0.0),
        },
        // Right
        Vertex {
            pos: glm::vec3(0.5, -0.5, 0.0),
        },
        // Up
        Vertex {
            pos: glm::vec3(-0.5, 0.5, 0.0),
        },
    ));

    // screen_world.add_tri(TriangleMesh(
    //     Vertex {
    //         pos: glm::vec3(0.5, -0.5, 0.0),
    //     },
    //     Vertex {
    //         pos: glm::vec3(0.5, 0.5, 0.0),
    //     },
    //     Vertex {
    //         pos: glm::vec3(-0.5, 0.5, 0.0),
    //     },
    // ));
    let mut sleep_passed = false;

    'going: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event as Ev;
            match event {
                Ev::Quit { .. }
                | Ev::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => {
                    break 'going;
                }
                Ev::Window {
                    timestamp,
                    window_id,
                    win_event: WindowEvent::Resized(width, height),
                } if window_id == main_id => {
                    if sleep_passed {
                        sleep_passed = false;
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                    }
                }
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            render_ctx.render_world(&screen_world).unwrap();
            render_ui();
        }
        window.gl_swap_window();
        let now = SystemTime::now();
        println!(
            "Now: {}",
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        thread::sleep(Duration::from_millis(100));
        sleep_passed = true;
    }
}

fn render_ui() {
    ()
}
