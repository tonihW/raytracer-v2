use bvh::bvh::BVH;

use crate::{triangle::Triangle, material::Material, camera::Camera};

use std::collections::HashMap;

pub struct Scene {
    pub shapes: Vec<Triangle>,
    pub materials: HashMap<String, Material>,
    pub bvh: Option<BVH>,
    pub camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            shapes: Vec::new(),
            materials: HashMap::new(),
            bvh: None,
            camera,
        }
    }
}
