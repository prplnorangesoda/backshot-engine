use std::ops::Index;

use crate::vertex::Vertex;

use paste::paste;

#[derive(Clone, Debug)]
pub struct TriPlane(pub [Vertex; 3]);

#[derive(Clone, Debug)]
pub struct NGonPlane(pub Box<[Vertex]>);

/// A plane on a brush.
#[derive(Clone, Debug)]
pub enum BrushPlane {
    /// A plane with 3 vertices.
    Triangle(TriPlane),
    /// A plane with 4+ vertices.
    /// Plane must be convex.
    NGon(NGonPlane),
}
impl Index<usize> for BrushPlane {
    type Output = Vertex;
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Triangle(plane) => &plane.0[index],
            Self::NGon(plane) => &plane.0[index],
        }
    }
}
macro_rules! brush_decl {
    ($name:ident, $count:expr) => {
        paste! {
            /// An x-pointed prism in the world representing static geometry
            /// (walls, floor, etc.).
            /// If you look top-down, it would look like:
            /// 3-pointed: a triangle!
            /// 4-pointed: a square!
            #[derive(Clone, Debug)]
            pub struct [<Brush $count>] ([BrushPlane; $count + 2]);

            impl Brush for [<Brush $count>] {
              fn planes(&self) -> &[BrushPlane] {
                &self.0
              }
            }
        }
    };
}

// A Brush-like object that has renderable planes.
pub trait Brush {
    fn planes(&self) -> &[BrushPlane];
}

brush_decl!(Brush3, 3);
brush_decl!(Brush4, 4);
