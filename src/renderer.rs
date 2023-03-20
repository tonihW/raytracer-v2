use bvh::{bvh::BVH, ray::Ray};
use glam::{Vec3, Vec2};
use image::{GenericImageView, Pixel};
use std::collections::HashMap;

use crate::{
    intersection::Intersection,
    triangle::Triangle,
    utils::EPSILON,
    material::Material,
};

const RESULT_NULL: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const RAYTRACER_LIGHT: Vec3 = Vec3::new(0.3, -1.0, 0.3);
const RAYTRACER_AMBIENT: Vec3 = Vec3::new(0.3, 0.4, 0.4);

pub struct Raytracer;
pub struct Pathtracer;
pub enum Renderer {
    RAYTRACER(Raytracer),
    PATHTRACER(Pathtracer),
}

fn sample_texture<P: Pixel>(img: &dyn GenericImageView<Pixel = P>, tex: &Vec2) -> (f32, f32, f32, u8, u8) where P: Pixel<Subpixel = u8> + 'static {
    // get pixel sample at texture coordinate, wrap around width & height
    let p = img.get_pixel(
        (tex.x * img.width() as f32) as u32 % img.width(),
        (tex.y * img.height() as f32) as u32 % img.height(),
    );

    // return results based on pixel channel count
    match p.channels().len() {
        2 => {
            let p = p.to_luma_alpha();
            return (0.0, 0.0, 0.0, p[0], p[1]);
        },
        3 => {
            let p = p.to_rgb();
            return (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0, 255, 255);
        },
        4 => {
            let p = p.to_rgba();
            return (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0, 255, p[3]);
        },
        _ => return (0.0, 0.0, 0.0, 255, 255)
    }
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
                    let c = sample_texture(a_texture, &hit_result.tex);
                    if c.3 == 0 {
                        let n_ray = Ray::new(hit_result.pos, ray.direction);
                        return result + Raytracer::trace(bvh, shp, mts, &n_ray, n + 1);
                    }
                }
                
                // transparency via diffuse texture
                let mut d_color = hit_mat.diffuse;
                if !hit_mat.diffuse_texture.is_none() {
                    let d_texture = hit_mat.diffuse_texture.as_ref().unwrap();
                    let c = sample_texture(d_texture, &hit_result.tex);
                    if c.4 == 0 {
                        let n_ray = Ray::new(hit_result.pos, ray.direction);
                        return result + Raytracer::trace(bvh, shp, mts, &n_ray, n + 1);
                    }
                    d_color = Vec3::new(c.0, c.1, c.2);
                }
                
                // check if in shadow
                let l_ray = Ray::new(hit_result.pos + hit_result.nrm * EPSILON, -RAYTRACER_LIGHT.normalize());
                let l_hits = bvh.traverse(&l_ray, &shp);
                let mut l_hit_dist = f32::MAX;
                let mut l_hit_isect: Option<Intersection> = None;
                for l_hit in l_hits {
                    match l_hit.intersect(&l_ray) {
                        Some(l_hit_result) => {
                            if l_hit_result.t < l_hit_dist {
                                l_hit_dist = l_hit_result.t;
                                l_hit_isect = Some(l_hit_result);
                            }
                        },
                        None => (),
                    }
                }

                let mut l_shadow = false;
                match l_hit_isect {
                    Some(l_hit_result) => {
                        // in shadow by default
                        l_shadow = true;

                        // check for transparency
                        let l_hit_mat = mts.get(l_hit_result.mat).unwrap();
                        if !l_hit_mat.alpha_texture.is_none() {
                            // transparency via alpha texture
                            let a_texture = l_hit_mat.alpha_texture.as_ref().unwrap();
                            let c = sample_texture(a_texture, &l_hit_result.tex);
                            if c.3 == 0 {
                                l_shadow = false;
                            }
                        } else if !l_hit_mat.diffuse_texture.is_none() {
                            // transparency via diffuse texture
                            let d_texture = l_hit_mat.diffuse_texture.as_ref().unwrap();
                            let c = sample_texture(d_texture, &l_hit_result.tex);
                            if c.4 == 0 {
                                l_shadow = false;
                            }
                        }
                    },
                    None => (),
                }
                
                // shade if not in shadow
                if !l_shadow {
                    let n_dot_l = hit_result.nrm.dot(-RAYTRACER_LIGHT.normalize());
                    result += n_dot_l * d_color;
                }

                // ambient light
                result += RAYTRACER_AMBIENT * d_color;

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
