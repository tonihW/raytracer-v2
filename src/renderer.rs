use bvh::{bvh::BVH, ray::Ray};
use glam::Vec3;

use crate::{
    intersection::Intersection,
    triangle::Triangle,
    utils::EPSILON,
};

const RESULT_NULL: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const RAYTRACER_LIGHT: Vec3 = Vec3::new(-0.8, -0.5, 0.1);

pub struct Raytracer;
pub struct Pathtracer;
pub enum Renderer {
    RAYTRACER(Raytracer),
    PATHTRACER(Pathtracer),
}

impl Raytracer {
    pub fn trace(bvh: &BVH, shp: &Vec<Triangle>, ray: &Ray, n: u8) -> Vec3 {
        // limit recursion
        if n > 4 {
            return RESULT_NULL;
        }

        // find closest intersection
        let hits = bvh.traverse(&ray, &shp);
        let mut hit_dist = f32::MAX;
        let mut hit_isect: Option<Intersection> = None;
        for hit in hits {
            match hit.intersect(&ray) {
                Some(hit_result) => {
                    if hit_result.t < hit_dist {
                        hit_dist = hit_result.t;
                        hit_isect = Some(hit_result);
                    }
                },
                None => (),
            }
        }

        // calculate shading
        let mut result = RESULT_NULL;
        match hit_isect {
            Some(hit_result) => {
                // check if in shadow
                let l_ray = Ray::new(hit_result.pos + hit_result.nrm * EPSILON, -RAYTRACER_LIGHT.normalize());
                let l_hits = bvh.traverse(&l_ray, &shp);
                let mut l_shadow = false;
                for l_hit in l_hits {
                    match l_hit.intersect(&l_ray) {
                        Some(_) => {
                            l_shadow = true;
                            break;
                        },
                        None => (),
                    }
                }
                
                // shade if not in shadow
                if !l_shadow {
                    let n_dot_l = hit_result.nrm.dot(-RAYTRACER_LIGHT.normalize());
                    result += n_dot_l * hit_result.mat.diffuse + hit_result.mat.emission;
                }
            },
            None => (),
        }
        
        return result;
    }
}
