use std::{thread, time::Duration};

use render::Render;
use sdl2::keyboard::Keycode;

mod render;
mod vector3;
mod vertex;

use vector3::Vector3;
use vertex::{TriangleMesh, Vertex};

#[derive(Default, Debug)]
struct WorldMesh {
    triangles: Vec<vertex::TriangleMesh>,
}

impl WorldMesh {
    fn add_tri(&mut self, tri: vertex::TriangleMesh) {
        self.triangles.push(tri);
    }
}

struct Camera {
    pos: Vector3,
    /// XYZ Euler angles. (0,0,0) means upwards.
    /// X: Roll
    /// Y: Pitch
    /// Z: Yaw
    orientation: Vector3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            orientation: Vector3::xyz(-90.0, 0.0, 0.0),
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
        .opengl()
        .build()
        .unwrap();
    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    let gl_ctx = window.gl_create_context().unwrap();
    let mut render_ctx = Render::init(&gl_ctx);

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut world = WorldMesh::default();
    let mut _camera = Camera::default();
    world.add_tri(TriangleMesh(
        // Left
        Vertex {
            pos: Vector3 {
                x: -0.5,
                y: -0.5,
                z: 0.0,
            },
        },
        // Right
        Vertex {
            pos: Vector3 {
                x: 0.5,
                y: -0.5,
                z: 0.0,
            },
        },
        // Up
        Vertex {
            pos: Vector3 {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
        },
    ));

    world.add_tri(TriangleMesh(
        Vertex {
            pos: Vector3::xyz(0.5, -0.7, 0.2),
        },
        Vertex {
            pos: Vector3::xyz(0.6, -0.5, 0.2),
        },
        Vertex {
            pos: Vector3::xyz(-0.2, -0.8, 0.2),
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
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            render_ctx.render_world(&world).unwrap();
            render_ui();
        }
        window.gl_swap_window();
        thread::sleep(Duration::from_millis(100));
    }
}

fn render_ui() {
    ()
}
