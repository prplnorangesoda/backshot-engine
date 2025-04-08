use std::ffi::CString;

use super::shader::{CompiledShader, Shader};

pub struct Program {
    id: gl::types::GLuint,
}

pub struct ProgramArgs<'a> {
    pub vert_shader: &'a CompiledShader,
    pub geo_shader: Option<&'a CompiledShader>,
    pub frag_shader: &'a CompiledShader,
    pub extra_shaders: &'a [&'a CompiledShader],
}

macro_rules! construct_program {
    ($vert_sh:expr, $geo_shader:expr, $frag_shader:expr; $($any_extra_shader:expr),*) => {
        compile_error!("Not implemented");
    };
    ($vert_sh:expr, $frag_shader:expr; $($any_extra_shader:expr),*) => {{
        let args = crate::gl_wrappers::program::ProgramArgs {
            vert_shader: &$vert_sh,
            geo_shader: None,
            frag_shader: &$frag_shader,
            extra_shaders: &[
                $(&$any_extra_shader),*
            ]
        };
        Program::from_args(args)
    }};
}

impl Program {
    pub fn new(
        vert_shader: &CompiledShader,
        geo_shader: Option<&CompiledShader>,
        frag_shader: &CompiledShader,
    ) -> Result<Self, String> {
        Self::from_args(ProgramArgs {
            frag_shader,
            vert_shader,
            geo_shader,
            extra_shaders: &[],
        })
    }
    pub fn from_args(args: ProgramArgs<'_>) -> Result<Self, String> {
        let inner = unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, args.vert_shader.id());
            gl::AttachShader(program, args.frag_shader.id());
            if let Some(shader) = args.geo_shader {
                gl::AttachShader(program, shader.id());
            }
            for shader in args.extra_shaders.iter() {
                gl::AttachShader(program, shader.id());
            }
            ::gl::LinkProgram(program);
            let mut success = 0;
            ::gl::GetProgramiv(program, ::gl::LINK_STATUS, &mut success);
            if success != gl::TRUE.into() {
                let mut infolog: Vec<u8> = vec![0; 512];
                let mut length = 0;
                ::gl::GetProgramInfoLog(program, 512, &mut length, infolog.as_mut_ptr().cast());
                infolog.truncate(length.try_into().unwrap());
                let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
                return Err(format!(
                    "Error linking program. OpenGL reply: '{}'",
                    loggable_string
                ));
            }
            program
        };

        Ok(Self { id: inner })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}
