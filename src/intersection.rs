use glam::{Vec3, Vec2};

use crate::material::Material;

#[derive(Debug)]
pub struct Intersection {
    pub t: f32,
    pub pos: Vec3,
    pub nrm: Vec3,
    pub tex: Vec2,
    pub mat: Material,
}
