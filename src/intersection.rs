use glam::{Vec3, Vec2};

#[derive(Debug)]
pub struct Intersection {
    t: f32,
    pos: Vec3,
    nrm: Vec3,
    tex: Vec2,
}
