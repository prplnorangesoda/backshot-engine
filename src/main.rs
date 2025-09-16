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
mod ui;
mod vector3;
mod vertex;
mod world;

use vertex::Vertex;
use world::World;

use crate::{
    brush::{BrushPlane, NGonPlane, TriPlane},
    gl_wrappers::gl_upd_viewport,
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

const START_WIDTH: u32 = 800;
const START_HEIGHT: u32 = 600;

/// The actual planes rendered to the screen.
#[derive(Debug)]
struct ScreenSpaceMesh {
    planes: Vec<BrushPlane>,
}

impl ScreenSpaceMesh {
    fn new() -> Self {
        Self { planes: vec![] }
    }
    fn add_tri(&mut self, tri: TriPlane) {
        self.planes.push(BrushPlane::Triangle(tri));
    }
    fn add_ngon(&mut self, ngon: NGonPlane) {
        self.planes.push(BrushPlane::NGon(ngon))
    }

    fn clear(&mut self) {
        self.planes.clear();
    }

    // create an example with a triangle
    fn simple() -> Self {
        let mut ret = Self::new();
        ret.add_tri(TriPlane([
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
        ret
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
    _source: gl::types::GLenum,
    _output_type: gl::types::GLenum,
    _id: gl::types::GLuint,
    _severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    _message: *const c_char,
    _user_param: *mut c_void,
) {
    println!("Debug output called")
}

fn main() {
    let (sdl_ctx, video_ctx, window, main_id) = init_sdl().unwrap();
    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let gl_ctx = window.gl_create_context().unwrap();

    gl_upd_viewport(START_WIDTH, START_HEIGHT);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_output), null());
    }

    let mut render_ctx = Render::init(&gl_ctx);

    render_ctx.clear().unwrap();
    window.gl_swap_window();

    let (mut imgui, mut imgui_platform, mut imgui_renderer) = imgui_create();

    let mut world = World::new();
    eprintln!("World: 0x{:x}", (&raw const world).addr());
    let mut _camera = Camera::default();

    let mut screen_world = ScreenSpaceMesh::simple();

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
    let mut last_debug_check = Instant::now();

    let mut frame_width: u32 = START_WIDTH;
    let mut frame_height: u32 = START_HEIGHT;

    // how much time last frame took to render
    let mut delta_time = 0.;
    'going: loop {
        let instant_loop_start = Instant::now();
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
                    frame_width = width.try_into().unwrap();
                    frame_height = height.try_into().unwrap();
                }
                _ => {}
            }
        }

        gl_upd_viewport(frame_width, frame_height);
        render_ctx.clear().unwrap();
        render_ctx.render_world(&screen_world).unwrap();

        imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);
        let draw_data = create_ui(&mut imgui);

        imgui_renderer.render(draw_data);

        window.gl_swap_window();

        let instant_before_sleep = Instant::now();
        // Soft cap fps
        thread::sleep(
            DURATION_BETWEEN_FRAMES
                .checked_sub(instant_before_sleep.duration_since(instant_loop_start))
                .unwrap_or(Duration::ZERO),
        );
        let instant_after_sleep = Instant::now();

        let frametime = instant_after_sleep
            .duration_since(instant_loop_start)
            .as_secs_f64();
        frametime_collector.push(frametime);

        // If it's been over a second since
        // last debug print, print it
        if instant_after_sleep
            .duration_since(last_debug_check)
            .as_secs()
            >= 1
        {
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
            last_debug_check = Instant::now();
        }

        frame_count += 1;
        delta_time = Instant::now()
            .duration_since(instant_loop_start)
            .as_secs_f64();
    }
}

fn init_sdl() -> Result<(sdl2::Sdl, sdl2::VideoSubsystem, video::Window, u32), String> {
    let sdl_ctx = sdl2::init()?;

    let video_ctx = sdl_ctx.video()?;
    video_ctx.gl_load_library_default()?;

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
        .map_err(|_| String::from(concat!("Error creating window. {} {}", file!(), line!())))?;

    let main_id = window.id();
    Ok((sdl_ctx, video_ctx, window, main_id))
}

fn imgui_create() -> (Context, ImguiSdlPlatform, ImguiRenderer) {
    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let imgui_platform = ImguiSdlPlatform::new(&mut imgui);
    let imgui_renderer = ImguiRenderer::new(&mut imgui);
    (imgui, imgui_platform, imgui_renderer)
}

fn create_ui(imgui: &mut Context) -> &DrawData {
    imgui.render()
}
