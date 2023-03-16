use glam::Vec3;
use image::{RgbaImage, GrayAlphaImage};

#[derive(Debug, Clone)]
pub struct Material {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub shininess: f32,
    pub emission: Vec3,
    pub diffuse_texture: Option<RgbaImage>,
    pub alpha_texture: Option<GrayAlphaImage>,
}
