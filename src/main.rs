#![allow(dead_code, clippy::let_and_return)]
use std::{
    thread,
    time::{Duration, Instant},
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

// Change this to change all values related to framecapping!
// Note: this is SOFT due to the fact that we may or may not sleep
// less, since we do calculations to not over-sleep, which may not
// be perfect because for some reason keeping time is difficult
const SOFT_FPS_CAP: u64 = 10;

const MAX_MICROS_BETWEEN_FRAMES: u64 = 1000000 / SOFT_FPS_CAP;
const MAX_MILLIS_BETWEEN_FRAMES: u64 = MAX_MICROS_BETWEEN_FRAMES / 1000;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(MAX_MICROS_BETWEEN_FRAMES);
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
        .window("SDL world test", 800, 600)
        .position_centered()
        // .resizable()
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

    let mut world = World::new();
    eprintln!("World: 0x{:x}", (&raw const world).addr());
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
    let mut updated_opengl_viewport_this_frame;

    let mut frame_count: u64 = 1;

    let mut frametime_collector = Vec::with_capacity(SOFT_FPS_CAP.try_into().unwrap());
    let mut last_frametime_check = Instant::now();
    'going: loop {
        let time_before_render = Instant::now();
        updated_opengl_viewport_this_frame = false;
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
                    timestamp: _,
                    window_id,
                    win_event: WindowEvent::Resized(width, height),
                } if window_id == main_id => {
                    // Only resize the opengl window once per update
                    if !updated_opengl_viewport_this_frame {
                        updated_opengl_viewport_this_frame = true;
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                    }
                }
                _ => {}
            }
        }

        render_ctx.clear().unwrap();
        render_ctx.render_world(&screen_world).unwrap();
        render_ui();

        window.gl_swap_window();

        let before_sleep = Instant::now();
        // Hard cap fps
        thread::sleep(DURATION_BETWEEN_FRAMES - before_sleep.duration_since(time_before_render));
        let now = Instant::now();
        let seconds = now.duration_since(before_sleep).as_secs_f64();
        frametime_collector.push(seconds);

        // If it's been over a second since
        // last debug print, print it
        if now.duration_since(last_frametime_check).as_secs() >= 1 {
            // can't reduce since we're keeping this Vec around
            let total_time = frametime_collector.iter().fold(0., |acc, item| acc + *item);
            // i love you rust but why can't i turbofish into()
            let len_int32: i32 = frametime_collector.len().try_into().unwrap();
            let len_float: f64 = len_int32.into();
            let avg_time: f64 = total_time / len_float;
            eprintln!("Average frametime (in seconds) over last second: {avg_time:0.5}");

            frametime_collector.clear();
            last_frametime_check = Instant::now();
        }

        frame_count += 1;
    }
}

fn render_ui() {}
