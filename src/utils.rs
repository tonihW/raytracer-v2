use glam::Vec3;

pub const EPSILON: f32 = 1e-5;

pub fn reflect(incoming: &Vec3, normal: &Vec3) -> Vec3 {
    return *incoming - (*normal * normal.dot(*incoming) * 2.0);
}
