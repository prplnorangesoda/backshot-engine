use crate::vector3::Vector3;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub pos: Vector3,
}

#[derive(Clone, Debug)]
pub struct TriangleMesh(pub Vertex, pub Vertex, pub Vertex);
