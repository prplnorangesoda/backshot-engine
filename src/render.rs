use std::{
    ffi::{c_void, CStr, CString},
    ptr::null,
};

use gl::types as gltype;
use sdl2::video::GLContext;

use crate::WorldMesh;

pub struct Render {
    vbo: gltype::GLuint,
    vao: gltype::GLuint,
    program: gltype::GLuint,
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

macro_rules! link_program {
    ($($shader:expr),+) => {unsafe {
        #![allow(unused_unsafe)]
        let program = ::gl::CreateProgram();
        let mut ret = Ok(program);
        $(
        ::gl::AttachShader(program, $shader);


        )*
        ::gl::LinkProgram(program);
        let mut success = 0;
        ::gl::GetProgramiv(program, ::gl::LINK_STATUS, &mut success);
        if success != gl::TRUE.into() {
            let mut infolog: Vec<u8> = vec![0; 512];
            let mut length = 0;
            ::gl::GetProgramInfoLog(program, 512, &mut length, infolog.as_mut_ptr().cast());
            infolog.truncate(length.try_into().unwrap());
            let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
            ret = Err(format!("Error linking program: {}", loggable_string));
        } else {
            $(
                ::gl::DeleteShader($shader);
            )*

        }
        ret
    }
};
}
impl Render {
    pub fn init(gl_ctx: &GLContext) -> Self {
        assert!(
            gl_ctx.is_current(),
            "gl_ctx must be current in order to create a Render"
        );
        let (vao, vbo, program) = unsafe {
            if INITIALIZED_ALREADY {
                panic!("Cannot initialize this more than once");
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

            let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
            compile_shader(vert_shader, VERT_SHADER_SOURCE).unwrap();

            let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            compile_shader(frag_shader, FRAG_SHADER_SOURCE).unwrap();

            let program = link_program!(vert_shader, frag_shader).unwrap();

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            (vao, vbo, program)
        };

        Render { vao, vbo, program }
    }

    pub fn render_world(&mut self, world: &WorldMesh) -> Result<(), ()> {
        let mut vertex_arr: Vec<f64> = Vec::with_capacity(world.triangles.len() * 9);
        dbg!(&world);
        let mut value = 1.0;
        for tri in world.triangles.iter() {
            let tri = tri.clone();
            value -= 0.05;
            push_vertex_to_vec!(vertex_arr, tri.0);
            vertex_arr.extend_from_slice(&[value; 3]);
            value -= 0.05;
            push_vertex_to_vec!(vertex_arr, tri.1);
            vertex_arr.extend_from_slice(&[value; 3]);
            value -= 0.05;
            push_vertex_to_vec!(vertex_arr, tri.2);
            vertex_arr.extend_from_slice(&[value; 3]);
        }
        dbg!(&vertex_arr);
        unsafe {
            let mut arr = vertex_arr.into_boxed_slice();
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::NamedBufferData(
                self.vbo,
                (arr.len() * std::mem::size_of::<f64>()).try_into().unwrap(),
                arr.as_mut_ptr().cast(),
                gl::DYNAMIC_DRAW,
            );
            println!("{}", gl::GetError());
            gl::DrawArrays(gl::TRIANGLES, 0, (arr.len() / 3).try_into().unwrap());
            println!("{}", gl::GetError());
        }
        Ok(())
    }
}

pub fn compile_shader(
    shader: gltype::GLuint,
    shader_source: &CStr,
) -> Result<gltype::GLuint, String> {
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
    Ok(0)
}
