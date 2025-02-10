use std::{thread, time::Duration};

use sdl2::keyboard::Keycode;

mod vertex;

#[derive(Default)]
struct WorldMesh {
    triangles: Vec<vertex::TriangleMesh>,
}

impl WorldMesh {
    fn add_tri(&mut self, tri: vertex::TriangleMesh) {
        self.triangles.push(tri);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
struct Camera {
    pos: Vector3,
    orientation: Vector3,
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
    video_ctx.gl_attr().set_context_major_version(3);
    video_ctx.gl_attr().set_context_minor_version(3);
    let window = video_ctx
        .window("SDL", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    let gl_ctx = window.gl_create_context().unwrap();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut _world = WorldMesh::default();

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
            render_world();
            render_ui();
        }
        window.gl_swap_window();
        thread::sleep(Duration::from_millis(100));
    }
}

fn render_world() {
    ()
}

fn render_ui() {
    ()
}
