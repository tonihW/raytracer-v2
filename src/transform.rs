use glam::{Vec3, Quat};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub pos: Vec3,
    pub ori: Quat,
    pub scl: Vec3,
}

impl Transform {
    pub fn new(pos: Vec3, ori: Quat, scl: Vec3) -> Transform {
        Transform {
            pos,
            ori,
            scl,
        }
    }

    pub fn from_axis_angle(pos: Vec3, axis: Vec3, angle: f32) -> Transform {
        Transform {
            pos,
            ori: Quat::from_axis_angle(axis, angle),
            scl: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}
