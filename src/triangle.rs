use crate::intersection::Intersection;
use crate::vertex::Vertex;
use bvh::ray::Ray;

pub struct Triangle {
    vrt: [Vertex; 3],
}

impl Triangle {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        
    }
}


