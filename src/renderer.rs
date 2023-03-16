use bvh::{bvh::BVH, ray::Ray};
use glam::Vec3;
use std::collections::HashMap;

use crate::{
    intersection::Intersection,
    triangle::Triangle,
    utils::EPSILON,
    material::Material,
};

const RESULT_NULL: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const RAYTRACER_LIGHT: Vec3 = Vec3::new(-0.8, -0.5, 0.1);
const RAYTRACER_AMBIENT: Vec3 = Vec3::new(0.3, 0.3, 0.33);

pub struct Raytracer;
pub struct Pathtracer;
pub enum Renderer {
    RAYTRACER(Raytracer),
    PATHTRACER(Pathtracer),
}

impl Raytracer {
    pub fn trace(bvh: &BVH, shp: &Vec<Triangle>, mts: &HashMap<String, Material>, ray: &Ray, n: u8) -> Vec3 {
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
                // get reference to material
                let hit_mat = mts.get(hit_result.mat).unwrap();
                
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
                
                // determine diffuse color
                let mut d_color = hit_mat.diffuse;
                if !hit_mat.diffuse_texture.is_none() {
                    let d_texture = hit_mat.diffuse_texture.as_ref().unwrap();
                    let d_texture_color = d_texture
                        .get_pixel(
                            (hit_result.tex.x * d_texture.width() as f32) as u32,
                            (hit_result.tex.y * d_texture.height() as f32) as u32
                        );
                    d_color = Vec3::new(d_texture_color[0] as f32 / 255.0, d_texture_color[1] as f32 / 255.0, d_texture_color[2] as f32 / 255.0);
                }
                
                // shade if not in shadow
                if !l_shadow {
                    let n_dot_l = hit_result.nrm.dot(-RAYTRACER_LIGHT.normalize());
                    result += n_dot_l * d_color;
                }

                // ambient light
                result += RAYTRACER_AMBIENT * hit_mat.ambient * d_color;

                // emissive light
                result += hit_mat.emission;
            },
            None => {
                let r_dot_l = ray.direction.dot(-RAYTRACER_LIGHT.normalize());
                result += RAYTRACER_AMBIENT + r_dot_l * RAYTRACER_AMBIENT;
            },
        }
        
        return result;
    }
}
