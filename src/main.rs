use std::{thread, time::Duration};

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

use vector3::Vector3_32;
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
    pos: Vector3_32,
    /// XYZ Euler angles. (0,0,0) means upwards.
    /// X: Roll
    /// Y: Pitch
    /// Z: Yaw
    orientation: Vector3_32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            orientation: Vector3_32::xyz(-90.0, 0.0, 0.0),
            pos: Default::default(),
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
            pos: Vector3_32::xyz(-0.5, -0.5, 0.0),
        },
        // Right
        Vertex {
            pos: Vector3_32::xyz(0.5, -0.5, 0.0),
        },
        // Up
        Vertex {
            pos: Vector3_32::xyz(-0.5, 0.5, 0.0),
        },
    ));

    screen_world.add_tri(TriangleMesh(
        Vertex {
            pos: Vector3_32::xyz(0.5, -0.5, 0.0),
        },
        Vertex {
            pos: Vector3_32::xyz(0.5, 0.5, 0.0),
        },
        Vertex {
            pos: Vector3_32::xyz(-0.5, 0.5, 0.0),
        },
    ));

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
                } => unsafe {
                    gl::Viewport(0, 0, width, height);
                },
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
        thread::sleep(Duration::from_millis(100));
    }
}

fn render_ui() {
    ()
}
