use std::{ffi::c_void, marker::PhantomData};

///
/// # Safety
/// You must ensure that as_gl_types and gl_type_layout match each other in terms of byte layout.
pub unsafe trait GlLayout {
    fn gl_type_layout() -> Box<[GlType]>;

    fn as_gl_bytes(&self) -> Box<[u8]>;
}

#[derive(Clone, Copy, Debug)]
pub enum GlType {
    Float,  // f32
    Double, // f64
}

impl GlType {
    pub fn get_size(&self) -> usize {
        match *self {
            GlType::Double => std::mem::size_of::<f64>(),
            GlType::Float => std::mem::size_of::<f32>(),
        }
    }
}
#[derive(Clone)]
pub struct RenderVec<T: GlLayout> {
    inner: Vec<u8>,
    layout: Box<[GlType]>,
    stride: usize,
    _phantom: PhantomData<T>,
}

impl<T: GlLayout> RenderVec<T> {
    pub fn new() -> Self {
        let layout = T::gl_type_layout();

        let mut stride = 0;
        for gl_type in layout.iter() {
            stride += gl_type.get_size();
        }
        println!("RenderVec stride: {stride}");
        Self {
            inner: vec![],
            layout,
            stride,
            _phantom: PhantomData,
        }
    }
    pub fn push(&mut self, value: T) {
        self.inner.extend_from_slice(&value.as_gl_bytes());
    }
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.inner.reserve(slice.len() * self.stride);
        for value in slice {
            self.inner.extend_from_slice(&value.as_gl_bytes());
        }
    }
    pub fn stride(&self) -> usize {
        self.stride
    }
    pub fn gl_size(&self) -> isize {
        (self.inner.len()).try_into().unwrap()
    }
    pub fn gl_len(&self) -> i32 {
        (self.inner.len() / self.stride).try_into().unwrap()
    }
    pub fn gl_data(&self) -> *const c_void {
        self.inner.as_ptr().cast()
    }
}
