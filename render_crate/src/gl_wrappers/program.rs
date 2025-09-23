//! Exports [`Program`].
use std::ffi::CString;

use super::CompiledShader;

/// Wrapper for an OpenGL program.
///
/// <https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects>
pub struct Program {
    /// The internal OpenGL id for this object.
    id: gl::types::GLuint,
}

/// Make a new [`Program`].
///
/// # Usage
/// ```no_run
/// let vert_shader = Shader::vertex(vertex_source_code).compile().unwrap();
/// let frag_shader = Shader::fragment(frag_source_code).compile().unwrap();
/// construct_program!(vert_shader, frag_shader;);
/// ```
#[macro_export]
macro_rules! construct_program {
    ($vert_sh:expr, $geo_shader:expr, $frag_shader:expr; $($any_extra_shader:expr),*) => {
        compile_error!("Not implemented");
    };
    ($vert_sh:expr, $frag_shader:expr; $($any_extra_shader:expr),*) => {{
        let args = $crate::gl_wrappers::program::ProgramArgs {
            vert_shader: &$vert_sh,
            geo_shader: None,
            frag_shader: &$frag_shader,
            extra_shaders: &[
                $(&$any_extra_shader),*
            ]
        };
        $crate::gl_wrappers::program::Program::from_args(args)
    }};
}

/// Necessary shaders to create a [`Program`].
/// Use [`construct_program!`] to easily create one of these.
pub struct ProgramArgs<'a> {
    /// A vertex shader.
    pub vert_shader: &'a CompiledShader,
    /// An optional geometry shader.
    pub geo_shader: Option<&'a CompiledShader>,
    /// A fragment shader.
    pub frag_shader: &'a CompiledShader,
    /// Any extra shaders that may be used by other shaders in this program.
    pub extra_shaders: &'a [&'a CompiledShader],
}

impl Program {
    /// Create a new program directly.
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
    /// Create a new program from a [`ProgramArgs`] struct.
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
    /// Get the internal id of this program.
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    /// Get the location of a uniform in this program.
    ///
    /// # Panics
    /// This function panics if `name` contains interior nuls.
    pub fn get_uniform_location(&self, name: impl AsRef<str>) -> Option<gl::types::GLint> {
        let name = CString::new(name.as_ref()).unwrap();
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, name.as_ptr().cast());
            if uniform_location < 0 {
                None
            } else {
                Some(uniform_location)
            }
        }
    }
    /// Get the location of an attrib in this program.
    ///
    /// # Panics
    /// This function panics if `name` contains interior nuls.
    pub fn get_attrib_location(&self, name: impl AsRef<str>) -> Option<gl::types::GLint> {
        let name = CString::new(name.as_ref()).unwrap();
        unsafe {
            let attrib_location = gl::GetAttribLocation(self.id, name.as_ptr().cast());
            if attrib_location < 0 {
                None
            } else {
                Some(attrib_location)
            }
        }
    }
}
