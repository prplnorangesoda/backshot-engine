#![allow(dead_code, clippy::let_and_return)]
use std::{
    ffi::{c_char, c_void},
    ptr::null,
    thread,
    time::{Duration, Instant},
};

use glm::Vec3;
use imgui::{Context, DrawData};
use imgui_sdl2_support::SdlPlatform as ImguiSdlPlatform;
use render::Render;
use sdl2::{event::WindowEvent, keyboard::Keycode, video};

mod brush;
mod entity;
mod imgui_wrappers;
#[macro_use]
mod gl_wrappers;
mod render;
mod vector3;
mod vertex;
mod world;

use vertex::Vertex;
use world::World;

use crate::{
    brush::{BrushPlane, NGonPlane, TriPlane},
    imgui_wrappers::renderer::ImguiRenderer,
};

/// This determines all values related to framecapping!
///
/// Note: this is SOFT due to the fact that we may or may not sleep
/// less, since we do calculations to not over-sleep, which may not
/// be perfect because for some reason keeping time is difficult
const SOFT_FPS_CAP: u64 = 1000;

const OPENGL_MAJOR_VER: u8 = 4;
const OPENGL_MINOR_VER: u8 = 3;

const MAX_MICROS_BETWEEN_FRAMES: u64 = 1_000_000 / SOFT_FPS_CAP;
const MAX_MILLIS_BETWEEN_FRAMES: u64 = MAX_MICROS_BETWEEN_FRAMES / 1000;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(MAX_MICROS_BETWEEN_FRAMES);

const START_WIDTH: i32 = 800;
const START_HEIGHT: i32 = 600;

/// The actual planes rendered to the screen.
#[derive(Default, Debug)]
struct ScreenSpaceMesh {
    planes: Vec<BrushPlane>,
}

impl ScreenSpaceMesh {
    fn add_tri(&mut self, tri: TriPlane) {
        self.planes.push(BrushPlane::Triangle(tri));
    }
    fn add_ngon(&mut self, ngon: NGonPlane) {
        self.planes.push(BrushPlane::NGon(ngon))
    }

    fn clear(&mut self) {
        self.planes.clear();
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

// void APIENTRY glDebugOutput(GLenum source, GLenum type, unsigned int id, GLenum severity,
//                            GLsizei length, const char *message, const void *userParam);

extern "system" fn gl_debug_output(
    source: gl::types::GLenum,
    output_type: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const c_char,
    user_param: *mut c_void,
) {
    println!("Debug output called")
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
    video_ctx
        .gl_attr()
        .set_context_major_version(OPENGL_MAJOR_VER);
    video_ctx
        .gl_attr()
        .set_context_minor_version(OPENGL_MINOR_VER);
    video_ctx
        .gl_attr()
        .set_context_profile(video::GLProfile::Core);

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
        gl::Viewport(0, 0, START_WIDTH, START_HEIGHT);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_output), null());
    }

    let gl_ctx = window.gl_create_context().unwrap();
    let mut render_ctx = Render::init(&gl_ctx);

    render_ctx.clear().unwrap();
    window.gl_swap_window();

    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut imgui_platform = ImguiSdlPlatform::new(&mut imgui);
    let mut imgui_renderer = ImguiRenderer::new(&mut imgui);

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut world = World::new();
    eprintln!("World: 0x{:x}", (&raw const world).addr());
    let mut _camera = Camera::default();

    let mut screen_world = ScreenSpaceMesh::default();
    screen_world.add_tri(TriPlane([
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
    ]));

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

    let mut frame_count: u64 = 1;

    let mut frametime_collector = Vec::with_capacity(SOFT_FPS_CAP as usize);
    let mut last_frametime_check = Instant::now();

    let mut frame_width = START_WIDTH;
    let mut frame_height = START_HEIGHT;

    'going: loop {
        let time_before_render = Instant::now();
        for event in event_pump.poll_iter() {
            imgui_platform.handle_event(&mut imgui, &event);
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
                    frame_width = width;
                    frame_height = height;
                }
                _ => {}
            }
        }

        unsafe {
            gl::Viewport(0, 0, frame_width, frame_height);
        }
        render_ctx.clear().unwrap();
        render_ctx.render_world(&screen_world).unwrap();

        imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);
        let draw_data = create_ui(&mut imgui);

        imgui_renderer.render(draw_data);

        window.gl_swap_window();

        let before_sleep = Instant::now();
        // Soft cap fps
        thread::sleep(
            DURATION_BETWEEN_FRAMES
                .checked_sub(before_sleep.duration_since(time_before_render))
                .unwrap_or(Duration::ZERO),
        );
        let now = Instant::now();
        let time_slept = now.duration_since(before_sleep).as_secs_f64();
        frametime_collector.push(now.duration_since(time_before_render).as_secs_f64());

        // If it's been over a second since
        // last debug print, print it
        if now.duration_since(last_frametime_check).as_secs() >= 1 {
            // can't reduce since we're keeping this Vec around
            let total_time = frametime_collector.iter().fold(0., |acc, item| acc + *item);
            let len_float: f64 = frametime_collector.len() as f64;
            let avg_time: f64 = total_time / len_float;
            eprintln!(
                "frametime: {avg_time:0.8}, FPS: {:0.8}, frames counted: {:05}",
                1. / avg_time,
                frametime_collector.len()
            );

            frametime_collector.clear();
            last_frametime_check = Instant::now();
        }

        frame_count += 1;
    }
}

fn create_ui(imgui: &mut Context) -> &DrawData {
    let ui = imgui.new_frame();
    ui.show_demo_window(&mut true);
    // ui.text("Hello");

    imgui.render()
}
