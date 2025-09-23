//! The rendering arm of the engine.
#![allow(dead_code, clippy::let_and_return)]
#![feature(generic_const_exprs, generic_const_items)]
#![allow(incomplete_features)]
#[cfg(doc)]
compile_error!("rustdoc does not support generic const expressions");

use std::{
    ffi::CStr,
    ops::Deref,
    ptr::{null, slice_from_raw_parts},
};

extern crate world;

pub mod gl_wrappers;
pub mod imgui_wrappers;
pub mod render_vec;
pub mod vector3;

pub use gl;
pub use gl_wrappers::gl_upd_viewport;
pub use glm;
pub use imgui;

use gl::types as gltype;
use glm::Vec3;
use sdl2::video::GLContext;
use world::{
    Vertex,
    brush::{BrushPlane, NGonPlane, TriPlane},
};

use crate::{
    gl_wrappers::{program::Program, shader::Shader},
    render_vec::{GlTypeList, RenderVec, StaticGlLayout},
    vector3::to_byte_slice,
};

/// The actual planes rendered to the screen.
#[derive(Debug)]
pub struct ScreenSpaceMesh {
    /// Internal vector of planes to be rendered.
    planes: Vec<BrushPlane>,
}

impl ScreenSpaceMesh {
    /// Create a new mesh.
    pub fn new() -> Self {
        Self { planes: vec![] }
    }
    /// Add a triangle to the internal vector.
    pub fn add_tri(&mut self, tri: TriPlane) {
        self.planes.push(BrushPlane::Triangle(tri));
    }
    /// Add an N-gon to the internal vector.
    pub fn add_ngon(&mut self, ngon: NGonPlane) {
        self.planes.push(BrushPlane::NGon(ngon))
    }

    /// Clears the internal vector, removing all polys.
    pub fn clear(&mut self) {
        self.planes.clear();
    }

    /// create this struct with one triangle
    pub fn simple() -> Self {
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
        ret
    }
}

/// A camera in the scene.
pub struct Camera {
    /// Position in world space.
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

pub struct Render {
    vbo: gltype::GLuint,
    vao: gltype::GLuint,
    program: Program,
}

macro_rules! include_cstr {
    ( $path:literal $(,)? ) => {{
        // Use a constant to force the verification to run at compile time.
        const VALUE: &'static ::core::ffi::CStr = match ::core::ffi::CStr::from_bytes_with_nul(
            concat!(include_str!($path), "\0").as_bytes(),
        ) {
            Ok(value) => value,
            Err(_) => panic!(concat!("interior NUL byte(s) in `", $path, "`")),
        };
        VALUE
    }};
}

const FRAG_SHADER_SOURCE: &CStr = include_cstr!("../glsl/frag_shader.glsl");
const VERT_SHADER_SOURCE: &CStr = include_cstr!("../glsl/vert_shader.glsl");

static mut INITIALIZED_ALREADY: bool = false;

// macro_rules! push_vertex_to_vec {
//     ($vec:expr, $vert:expr) => {{
//         use ::std::vec::Vec;
//         let vec: &mut Vec<_> = &mut $vec;
//         Vec::push(vec, $vert.pos.x);
//         Vec::push(vec, $vert.pos.y);
//         Vec::push(vec, $vert.pos.z);
//     }};
// }

// If you're adding something, ensure to update
// the GlLayout impl below!
#[repr(C)]
pub struct InputParams {
    position: Vec3,
    color: Vec3,
}
// TODO: fixme, this smells!
// don't want to dupe magic numbers, but making a constant unrelated to the type
// is REALLY BAD, and Rust doesn't allow const fns in traits, so we can't
// chuck the length logic in a trait and still use it in type sigs!
pub const INPUTPARAMS_TYPE_LENGTH: usize = 6;

unsafe impl StaticGlLayout for InputParams {
    const LEN: usize = INPUTPARAMS_TYPE_LENGTH;
    fn as_gl_bytes(&self) -> impl Deref<Target = [u8]> {
        let ret: &[u8] = unsafe {
            let floats: &[f32] = slice_from_raw_parts((self as *const InputParams).cast(), 6)
                .as_ref()
                .unwrap();
            to_byte_slice(floats)
        };
        ret
    }
    fn gl_type_layout() -> GlTypeList<INPUTPARAMS_TYPE_LENGTH> {
        let box_1 = Vec3::gl_type_layout();
        let box_2 = Vec3::gl_type_layout();

        GlTypeList::new([box_1[0], box_1[1], box_1[2], box_2[0], box_2[1], box_2[2]])
    }
}

impl Render {
    pub fn init(gl_ctx: &GLContext) -> Self {
        assert!(
            gl_ctx.is_current(),
            "gl_ctx must be current in order to create a Render"
        );
        let (vao, vbo, program) = unsafe {
            if INITIALIZED_ALREADY {
                panic!("Cannot initialize Render more than once");
            }
            INITIALIZED_ALREADY = true;

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);

            // bind the Vertex Array Object first, then bind and set vertex buffers, and then configure attributes
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            eprintln!("Render vbo: {vbo}");
            eprintln!("Render vao: {vao}");

            // we're NamedBufferData-ing this later when we need to use it
            gl::BufferData(gl::ARRAY_BUFFER, 0, null(), gl::DYNAMIC_DRAW);

            // position attrib
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * size_of::<f32>()).try_into().unwrap(),
                null(),
            );
            gl::EnableVertexAttribArray(0);

            // color attrib
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * size_of::<f32>()).try_into().unwrap(),
                (3 * size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
            // compile_shader(vert_shader, VERT_SHADER_SOURCE).unwrap();

            let vert_shader = Shader::vertex(VERT_SHADER_SOURCE.into()).compile().unwrap();

            // let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            // compile_shader(frag_shader, FRAG_SHADER_SOURCE).unwrap();

            let frag_shader = Shader::fragment(FRAG_SHADER_SOURCE.into())
                .compile()
                .unwrap();

            let program = construct_program!(vert_shader, frag_shader;).unwrap();

            // let program = link_program!(vert_shader, frag_shader).unwrap();
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

            // reset bound arrays
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            (vao, vbo, program)
        };

        Render { vao, vbo, program }
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        Ok(())
    }
    pub fn render_world(&mut self, world: &ScreenSpaceMesh) -> Result<(), ()> {
        let mut render_vec: RenderVec<InputParams> = RenderVec::new();
        // let mut vertex_arr: Vec<f64> = Vec::with_capacity(world.triangles.len() * 9);
        // dbg!(&world);
        // let colors = [0.584, 0.203, 0.92];
        // for tri in world.triangles.iter() {
        //     let tri = tri.clone();
        //     push_vertex_to_vec!(vertex_arr, tri.0);
        //     vertex_arr.extend_from_slice(&colors);
        //     push_vertex_to_vec!(vertex_arr, tri.1);
        //     vertex_arr.extend_from_slice(&colors);
        //     push_vertex_to_vec!(vertex_arr, tri.2);
        //     vertex_arr.extend_from_slice(&colors);
        // }
        let color = glm::vec3(0.584, 0.203, 0.92);
        for tri in world.planes.iter() {
            let plane = tri.clone();
            render_vec.push(InputParams {
                position: plane[0].pos,
                color,
            });
            render_vec.push(InputParams {
                position: plane[1].pos,
                color,
            });
            render_vec.push(InputParams {
                position: plane[2].pos,
                color,
            });
        }
        // dbg!(&vertex_arr);
        unsafe {
            // let mut arr = vertex_arr.into_boxed_slice();
            gl::UseProgram(self.program.id());
            gl::BindVertexArray(self.vao);
            // gl::NamedBufferData(
            //     self.vbo,
            //     (arr.len() * std::mem::size_of::<f64>()).try_into().unwrap(),
            //     arr.as_mut_ptr().cast(),
            //     gl::DYNAMIC_DRAW,
            // );
            gl::NamedBufferData(
                self.vbo,
                render_vec.gl_byte_size(),
                render_vec.gl_data(),
                gl::DYNAMIC_DRAW,
            );

            // if cfg!(debug_assertions) {
            //     let slice: &[f32] = std::slice::from_raw_parts(
            //         render_vec.gl_data().cast(),
            //         TryInto::<usize>::try_into(render_vec.gl_byte_size()).unwrap()
            //             / std::mem::size_of::<f32>(),
            //     );
            //     println!("slice for rendering: [");
            //     let mut iter = slice.chunks(3);
            //     let mut vertex_idx = 0;
            //     while let Some(pos) = iter.next() {
            //         let col = iter.next().unwrap();
            //         let byte_offset_idx = vertex_idx * (4 * size_of::<f32>());
            //         println!("vertex {vertex_idx:>2} (offset {byte_offset_idx:>4}): position: {pos: >16?}, colour: {col: >16?}");
            //         vertex_idx += 1;
            //     }
            //     println!("]");
            // }
            // let error_pre = gl::GetError() == gl::TRUE.into();
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::DrawArrays(gl::TRIANGLES, 0, render_vec.gl_len());

            gl::BindVertexArray(0);
            let error_post = gl::GetError() == gl::TRUE.into();
            if error_post {
                eprintln!("There was an error after gl::DrawArrays !!!");
            }
        }
        Ok(())
    }
}

// pub fn compile_shader(shader: gltype::GLuint, shader_source: &CStr) -> Result<(), String> {
//     unsafe {
//         gl::ShaderSource(shader, 1, &shader_source.as_ptr(), null());
//         gl::CompileShader(shader);

//         let mut success = 0;
//         gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

//         if success != gl::TRUE.into() {
//             let mut infolog: Vec<u8> = vec![0; 512];
//             let mut length = 0;
//             gl::GetShaderInfoLog(shader, 512, &mut length, infolog.as_mut_ptr().cast());
//             infolog.truncate(length.try_into().unwrap());
//             let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
//             return Err(format!("Internal opengl error: {}", loggable_string));
//         }
//     }
//     Ok(())
// }
