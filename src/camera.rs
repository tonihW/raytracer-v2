use crate::transform::Transform;
use glam::{Vec3, Quat};

#[derive(Debug)]
pub struct Camera {
    trf: Transform,
    viewport_w: f32,
    viewport_h: f32,
    viewport_a: f32,
}

impl Camera {
    pub fn new(trf: Transform, viewport_w: f32, viewport_h: f32) -> Camera {
        Camera {
            trf,
            viewport_w,
            viewport_h,
            viewport_a: viewport_h / viewport_w,
        }
    }

    pub fn from_axis_angle(pos: Vec3, axis: Vec3, angle: f32, viewport_w: f32, viewport_h: f32) -> Camera {
        Camera {
            trf: Transform::from_axis_angle(pos, axis, angle),
            viewport_w,
            viewport_h,
            viewport_a: viewport_h / viewport_w,
        }
    }

    pub fn calc_ray(&self, x: f32, y: f32) -> (Vec3, Vec3) {
        // calculate ray direction vector
        let x_norm = (self.viewport_w * 0.5 - x) / self.viewport_w;
        let y_norm = (self.viewport_h * 0.5 - y) / self.viewport_h * self.viewport_a;
        let v_norm = Vec3 {
            x: x_norm,
            y: y_norm,
            z: 1.0,
        };
        let q = &self.trf.ori;
        let q_inv = q.conjugate();
        let w = Quat::from_xyzw(v_norm.x, v_norm.y, v_norm.z, 0.0);
        let r = (*q * w * q_inv).normalize();

        (self.trf.pos, Vec3 {
            x: r.x,
            y: r.y,
            z: r.z,
        })
    }
}
