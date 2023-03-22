use glam::{Vec3, Quat};

use crate::utils::EPSILON;

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

    pub fn from_lookat(pos: Vec3, obj: Vec3) -> Transform {
        let forward = (obj - pos).normalize();
        let mut rot_axis = Vec3::Z.cross(forward).normalize();
        if rot_axis.length_squared() < EPSILON {
            rot_axis.x = 0.0;
            rot_axis.y = 1.0;
            rot_axis.z = 0.0;
        }
        let theta = Vec3::Z.dot(forward).acos();
        let ori = Quat::from_axis_angle(rot_axis, theta);
        
        Transform {
            pos,
            ori,
            scl: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}
