use glam::{Vec3, Vec2};

#[derive(Debug)]
pub struct Intersection<'a> {
    pub t: f32,
    pub pos: Vec3,
    pub nrm: Vec3,
    pub tex: Vec2,
    pub mat: &'a String,
}
