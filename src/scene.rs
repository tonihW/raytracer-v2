use bvh::bvh::BVH;
use glam::Vec3;

use crate::{triangle::Triangle, material::Material, camera::Camera, light::Light};

use std::collections::HashMap;

pub struct Scene {
    pub shapes: Vec<Triangle>,
    pub materials: HashMap<String, Material>,
    pub ambient: Vec3,
    pub lights: Vec<Box<dyn Light + Sync>>,
    pub bvh: Option<BVH>,
    pub camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            shapes: Vec::new(),
            materials: HashMap::new(),
            ambient: Vec3::new(0.0, 0.0, 0.0),
            lights: Vec::new(),
            bvh: None,
            camera,
        }
    }
}
