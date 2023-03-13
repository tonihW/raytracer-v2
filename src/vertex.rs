use glam::{Vec3, Vec2};

#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
    nrm: Vec3,
    tex: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec3, nrm: Vec3, tex: Vec2) -> Vertex {
        Vertex {
            pos,
            nrm,
            tex,
        }
    }
}

