use std::{ffi::CString, ptr::null};

pub struct Shader {
    inner: gl::types::GLuint,
    source: CString,
    was_compiled: bool,
}

pub enum ShaderType {
    Fragment,
    _Geometry,
    Vertex,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source: CString) -> Self {
        let shader = unsafe {
            match shader_type {
                ShaderType::Fragment => gl::CreateShader(gl::FRAGMENT_SHADER),
                ShaderType::_Geometry => gl::CreateShader(gl::GEOMETRY_SHADER),
                ShaderType::Vertex => gl::CreateShader(gl::VERTEX_SHADER),
            }
        };
        Self {
            inner: shader,
            source,
            was_compiled: false,
        }
    }
    pub fn vertex(source: CString) -> Self {
        Self::new(ShaderType::Vertex, source)
    }
    pub fn fragment(source: CString) -> Self {
        Self::new(ShaderType::Fragment, source)
    }
    pub fn geometry(source: CString) -> Self {
        Self::new(ShaderType::_Geometry, source)
    }

    pub fn compile(mut self) -> Result<CompiledShader, String> {
        let compiled_shader = unsafe {
            gl::ShaderSource(self.inner, 1, &self.source.as_ptr(), null());
            gl::CompileShader(self.inner);

            let mut success = 0;
            gl::GetShaderiv(self.inner, gl::COMPILE_STATUS, &mut success);

            if success != gl::TRUE.into() {
                let mut infolog: Vec<u8> = vec![0; 512];
                let mut length = 0;
                gl::GetShaderInfoLog(self.inner, 512, &mut length, infolog.as_mut_ptr().cast());
                infolog.truncate(length.try_into().unwrap());
                let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
                return Err(format!("Shader compilation error: {}", loggable_string));
            }
            self.was_compiled = true;
            self.inner
        };
        unsafe { Ok(CompiledShader::from_uint_unchecked(compiled_shader)) }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if !self.was_compiled {
                gl::DeleteShader(self.inner);
            }
        }
    }
}
impl Drop for CompiledShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        };
    }
}

pub struct CompiledShader {
    id: gl::types::GLuint,
}

impl CompiledShader {
    pub unsafe fn from_uint_unchecked(shader: gl::types::GLuint) -> Self {
        Self { id: shader }
    }
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}
