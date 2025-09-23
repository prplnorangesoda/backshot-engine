use core::fmt;
use std::{any::Any, ffi::c_void, marker::PhantomData, ops::Deref};

#[repr(transparent)]
#[derive(Clone)]
pub struct GlTypeList<const LEN: usize>(pub [GlType; LEN]);

impl<const LEN: usize> GlTypeList<LEN> {
    pub const fn new(list: [GlType; LEN]) -> Self {
        Self(list)
    }
}

impl<const LEN: usize> Deref for GlTypeList<LEN> {
    type Target = [GlType; LEN];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Refactored to explicitly be comptime with the LEN associated const.
///
/// Not dyn-compatible. Use [`DynamicGlLayout`] if that's what you're looking for.
/// This trait is built around static comptime optimizes.
/// # Safety
/// You must ensure that `as_gl_bytes` and `gl_type_layout` match each other in terms of byte layout.
/// If `gl_type_layout()` returns Float, Float, Float, `as_gl_bytes` must return a slice of 3 f32s.
///
pub unsafe trait StaticGlLayout {
    const LEN: usize;
    /// Returns the GL types that the bytes returned from calling [`as_gl_bytes`] on `self` will map to.
    ///
    /// [`as_gl_bytes`]: StaticGlLayout::as_gl_bytes
    fn gl_type_layout() -> GlTypeList<{ Self::LEN }>
    where
        [(); Self::LEN]:;

    /// Returns a byte slice for use in OpenGL rendering.
    ///
    /// The types map to what calling [`gl_type_layout`] on `self` would return.
    ///
    /// [`gl_type_layout`]: StaticGlLayout::gl_type_layout
    fn as_gl_bytes(&self) -> impl Deref<Target = [u8]>;
}

/// Dyn-compatible gl type layout.
///
/// # Safety
/// You must ensure that `as_gl_bytes` and `gl_type_layout` match each other in terms of byte layout.
/// If `gl_type_layout()` returns Float, Float, Float, `as_gl_bytes` must return a slice of 3 f32s.
pub unsafe trait DynamicGlLayout {
    /// Returns the GL types that the bytes returned from calling [`as_gl_bytes`] on this will map to.
    ///
    /// [`as_gl_bytes`]: StaticGlLayout::as_gl_bytes
    fn dyn_gl_type_layout(&self) -> Box<[GlType]>;

    /// Returns a wrapped type that derefs to a byte slice for use in OpenGL rendering.
    ///
    /// The types map to what calling [`gl_type_layout`] on `self` would return.
    ///
    /// ## Why Box\<dyn Deref>?
    /// For `Box::new([...])` and `Box::new(Rc::clone(...))` to both work.
    ///
    /// [`gl_type_layout`]: StaticGlLayout::gl_type_layout
    fn dyn_gl_bytes(&self) -> Box<dyn Deref<Target = [u8]>>;
}

unsafe impl<T> DynamicGlLayout for T
where
    [(); T::LEN]:,
    T: StaticGlLayout,
{
    fn dyn_gl_type_layout(&self) -> Box<[GlType]> {
        Box::new(T::gl_type_layout().0)
    }
    fn dyn_gl_bytes(&self) -> Box<dyn Deref<Target = [u8]>> {
        Box::new(self.as_gl_bytes().to_owned())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlType {
    Float,  // f32
    Double, // f64
}

impl GlType {
    pub const fn get_size(&self) -> usize {
        match *self {
            GlType::Double => std::mem::size_of::<f64>(),
            GlType::Float => std::mem::size_of::<f32>(),
        }
    }
}

// refactored to avoid allocations
#[derive(Clone)]
pub struct RenderVec<LAYOUT: StaticGlLayout> {
    inner: Vec<u8>,
    stride: usize,
    _phantom: PhantomData<LAYOUT>,
}

impl<LayoutT: StaticGlLayout> RenderVec<LayoutT>
where
    [(); LayoutT::LEN]:,
{
    pub fn new() -> Self {
        let layout = LayoutT::gl_type_layout();

        let mut stride = 0;
        for gl_type in layout.iter() {
            stride += gl_type.get_size();
        }
        // eprintln!("New RenderVec created, stride: {stride}");
        Self {
            inner: vec![],
            stride,
            _phantom: PhantomData,
        }
    }
    pub fn push(&mut self, value: LayoutT) {
        // dbg!("render_vec: pushing");
        self.inner.extend_from_slice(&value.as_gl_bytes());
        // dbg!(&self.inner);
    }
    pub fn extend_from_slice(&mut self, slice: &[LayoutT]) {
        self.inner.reserve(slice.len() * self.stride);
        for value in slice {
            self.inner.extend_from_slice(&value.as_gl_bytes());
        }
    }
    pub fn stride(&self) -> usize {
        self.stride
    }
    pub fn gl_byte_size(&self) -> isize {
        self.inner.len().try_into().unwrap()
    }
    pub fn gl_len(&self) -> i32 {
        (self.inner.len() / self.stride).try_into().unwrap()
    }
    pub fn gl_data(&self) -> *const c_void {
        self.inner.as_ptr().cast()
    }
}
