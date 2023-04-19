use glam::Vec3;
use image::{RgbaImage, GrayAlphaImage};

#[derive(Debug, Clone)]

pub enum TextureType {
    Diffuse,
    Alpha,
    None
}

pub enum Texture {
    Diffuse(RgbaImage),
    Alpha(GrayAlphaImage),
    None,
}

pub struct Material {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub shininess: f32,
    pub emission: Vec3,
    pub diffuse_texture: Texture,
    pub alpha_texture: Texture,
}

impl Material {
    pub fn brdf_lambertian(&self, normal: &Vec3, light: &Vec3) -> f32 {
        return normal.dot(*light);
    }

    pub fn brdf_phong(&self, reflect: &Vec3, view: &Vec3) -> f32 {
        return reflect.dot(*view).powf(self.shininess);
    }

    pub fn fresnel_schlick(&self, normal: &Vec3, view: &Vec3) -> f32 {
        return (1.0 - normal.dot(*view)).clamp(0.0, 1.0).powf(5.0);
    }
}
