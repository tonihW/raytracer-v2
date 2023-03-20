use bvh::{bvh::BVH, ray::Ray};
use glam::{Vec3, Vec2};
use image::{GenericImageView, Pixel};
use std::collections::HashMap;

use crate::{
    intersection::Intersection,
    triangle::Triangle,
    utils::{EPSILON, reflect},
    material::Material,
};

const RESULT_NULL: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const RAYTRACER_LIGHT: Vec3 = Vec3::new(-0.1, -1.0, 0.1);
const RAYTRACER_AMBIENT: Vec3 = Vec3::new(0.3, 0.4, 0.4);

pub struct Raytracer;
pub struct Pathtracer;
pub enum Renderer {
    RAYTRACER(Raytracer),
    PATHTRACER(Pathtracer),
}

fn sample_texture<P: Pixel>(img: &dyn GenericImageView<Pixel = P>, tex: &Vec2) -> (f32, f32, f32, u8, u8) where P: Pixel<Subpixel = u8> + 'static {
    // get pixel sample at texture coordinate, clamp to max width & height
    let pix_x = (tex.x * img.width() as f32) as u32;
    let pix_y = (tex.y * img.height() as f32) as u32;
    let max_w = img.width() - 1;
    let max_h = img.height() - 1;
    let pix_c = img.get_pixel(pix_x.clamp(0, max_w), pix_y.clamp(0, max_h));

    // return results based on pixel channel count
    match pix_c.channels().len() {
        2 => {
            let p = pix_c.to_luma_alpha();
            return (0.0, 0.0, 0.0, p[0], p[1]);
        },
        3 => {
            let p = pix_c.to_rgb();
            return (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0, 255, 255);
        },
        4 => {
            let p = pix_c.to_rgba();
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

                // pre-calc stuff
                let light = RAYTRACER_LIGHT.normalize();
                let reflection = reflect(&light, &hit_result.nrm).normalize();
                
                // apply shading
                if !l_shadow {
                    // diffuse
                    let brdf_d = hit_mat.brdf_lambertian(&hit_result.nrm, &-light);

                    // specular
                    let brdf_s = hit_mat.brdf_phong(&reflection, &-ray.direction);

                    result += d_color * brdf_d + d_color * brdf_s;
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
