use std::{
    ffi::{c_void, CStr, CString},
    ptr::null,
};

use gl::types as gltype;
use sdl2::video::GLContext;

use crate::{
    program::Program,
    render_vec::{GlLayout, RenderVec},
    shader::Shader,
    vector3::Vector3_32,
    WorldMesh,
};

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

macro_rules! push_vertex_to_vec {
    ($vec:expr, $vert:expr) => {{
        use ::std::vec::Vec;
        let vec: &mut Vec<_> = &mut $vec;
        Vec::push(vec, $vert.pos.x);
        Vec::push(vec, $vert.pos.y);
        Vec::push(vec, $vert.pos.z);
    }};
}

pub struct InputParams {
    position: Vector3_32,
    color: Vector3_32,
}

unsafe impl GlLayout for InputParams {
    fn as_gl_bytes(&self) -> Box<[u8]> {
        let box_1 = self.position.as_gl_bytes();
        let box_2 = self.color.as_gl_bytes();
        let mut vec = Vec::with_capacity(box_1.len() + box_2.len());
        vec.extend_from_slice(&box_1);
        vec.extend_from_slice(&box_2);
        let ret = vec.into_boxed_slice();

        println!("InputParams as_gl_bytes() box: {ret:?}");
        ret
    }
    fn gl_type_layout() -> Box<[crate::render_vec::GlType]> {
        let box_1 = Vector3_32::gl_type_layout();
        let box_2 = Vector3_32::gl_type_layout();
        let mut vec = Vec::with_capacity(box_1.len() + box_2.len());
        vec.extend_from_slice(&box_1);
        vec.extend_from_slice(&box_2);
        let ret = vec.into_boxed_slice();

        println!("InputParams gl_type_layout() box: {ret:?}");
        ret
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
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::BufferData(gl::ARRAY_BUFFER, 0, null(), gl::DYNAMIC_DRAW);

            gl::VertexAttribPointer(
                0,
                3,
                gl::DOUBLE,
                gl::FALSE,
                (6 * std::mem::size_of::<f64>()).try_into().unwrap(),
                null(),
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::DOUBLE,
                gl::FALSE,
                (6 * std::mem::size_of::<f64>()).try_into().unwrap(),
                (3 * std::mem::size_of::<f64>()) as *const _,
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

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            (vao, vbo, program)
        };

        Render { vao, vbo, program }
    }

    pub fn render_world(&mut self, world: &WorldMesh) -> Result<(), ()> {
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
        let colors = [0.584, 0.203, 0.92];
        for tri in world.triangles.iter() {
            let tri = tri.clone();
            render_vec.push(InputParams {
                position: tri.0.pos,
                color: colors.into(),
            });
        }
        // dbg!(&vertex_arr);
        unsafe {
            // let mut arr = vertex_arr.into_boxed_slice();
            gl::UseProgram(self.program.get_inner());
            gl::BindVertexArray(self.vao);
            // gl::NamedBufferData(
            //     self.vbo,
            //     (arr.len() * std::mem::size_of::<f64>()).try_into().unwrap(),
            //     arr.as_mut_ptr().cast(),
            //     gl::DYNAMIC_DRAW,
            // );
            gl::NamedBufferData(
                self.vbo,
                render_vec.gl_size(),
                render_vec.gl_data(),
                gl::DYNAMIC_DRAW,
            );
            let slice: &[f32] = std::slice::from_raw_parts(
                render_vec.gl_data().cast(),
                render_vec.gl_size().try_into().unwrap(),
            );
            println!("slice for rendering: {slice:?}");
            println!("Error (pre_drawarrays): {}", gl::GetError());
            gl::DrawArrays(gl::TRIANGLES, 0, render_vec.gl_len());
            println!("Error (post_drawarrays): {}", gl::GetError());
        }
        Ok(())
    }
}

pub fn compile_shader(shader: gltype::GLuint, shader_source: &CStr) -> Result<(), String> {
    unsafe {
        gl::ShaderSource(shader, 1, &shader_source.as_ptr(), null());
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success != gl::TRUE.into() {
            let mut infolog: Vec<u8> = vec![0; 512];
            let mut length = 0;
            gl::GetShaderInfoLog(shader, 512, &mut length, infolog.as_mut_ptr().cast());
            infolog.truncate(length.try_into().unwrap());
            let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
            return Err(format!("Internal opengl error: {}", loggable_string));
        }
    }
    Ok(())
}
