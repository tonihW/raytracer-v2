use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub shininess: f32,
    pub emission: Vec3,
}