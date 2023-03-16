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
const RAYTRACER_LIGHT: Vec3 = Vec3::new(-0.1, -1.0, 0.12);
const RAYTRACER_AMBIENT: Vec3 = Vec3::new(0.5, 0.5, 0.55);

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

                // transparency via alpha texture
                if !hit_mat.alpha_texture.is_none() {
                    let a_texture = hit_mat.alpha_texture.as_ref().unwrap();
                    let a_texture_color = a_texture
                        .get_pixel_checked(
                            (hit_result.tex.x * a_texture.width() as f32) as u32,
                            (hit_result.tex.y * a_texture.height() as f32) as u32
                        );
                    if !a_texture_color.is_none() {
                        let c = a_texture_color.unwrap();
                        if c[0] == 0 {
                            let n_ray = Ray::new(hit_result.pos, ray.direction);
                            return result + Raytracer::trace(bvh, shp, mts, &n_ray, n + 1);
                        }
                    }
                }
                
                // determine diffuse color
                let mut d_color = hit_mat.diffuse;
                let mut d_alpha: u8 = 0;
                if !hit_mat.diffuse_texture.is_none() {
                    let d_texture = hit_mat.diffuse_texture.as_ref().unwrap();
                    let d_texture_color = d_texture
                        .get_pixel_checked(
                            (hit_result.tex.x * d_texture.width() as f32) as u32,
                            (hit_result.tex.y * d_texture.height() as f32) as u32
                        );
                    if !d_texture_color.is_none() {
                        let c = d_texture_color.unwrap();
                        d_color = Vec3::new(c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
                        d_alpha = c[3];
                    }
                }

                // transparency via diffuse texture
                if d_alpha == 255 {
                    let n_ray = Ray::new(hit_result.pos, ray.direction);
                    return result + Raytracer::trace(bvh, shp, mts, &n_ray, n + 1);
                }
                
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
