use glam::Vec3;
use image::{RgbImage};

#[derive(Debug, Clone)]
pub struct Material {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub diffuse_texture: Option<RgbImage>,
    pub specular: Vec3,
    pub shininess: f32,
    pub emission: Vec3,
}
