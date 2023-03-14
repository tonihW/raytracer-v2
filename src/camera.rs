use crate::transform::Transform;
use bvh::ray::Ray;
use glam::{Vec3, Quat};

#[derive(Debug)]
pub struct Camera {
    pub trf: Transform,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub viewport_a: f32,
    pub film: Vec<f32>,
    pub film_samples: i32,
}

impl Camera {
    pub fn new(trf: Transform, viewport_w: f32, viewport_h: f32) -> Camera {
        Camera {
            trf,
            viewport_w,
            viewport_h,
            viewport_a: viewport_h / viewport_w,
            film: vec![0.0; viewport_w as usize * viewport_h as usize],
            film_samples: 0,
        }
    }

    pub fn from_axis_angle(pos: Vec3, axis: Vec3, angle: f32, viewport_w: f32, viewport_h: f32) -> Camera {
        Camera {
            trf: Transform::from_axis_angle(pos, axis, angle),
            viewport_w,
            viewport_h,
            viewport_a: viewport_h / viewport_w,
            film: vec![0.0; viewport_w as usize * viewport_h as usize],
            film_samples: 0,
        }
    }

    pub fn calc_ray(&self, x: f32, y: f32) -> Ray {
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

        return Ray::new(self.trf.pos, Vec3 {
            x: r.x,
            y: r.y,
            z: r.z,
        });
    }
}
