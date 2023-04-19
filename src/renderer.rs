use bvh::{ray::Ray};
use glam::{Vec3, Vec2, Vec4};
use image::{GenericImageView, Pixel};

use crate::{
    intersection::Intersection,
    utils::{EPSILON, reflect},
    scene::Scene, material::Texture,
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

/**
 * Reference: https://graphics.cg.uni-saarland.de/courses/cg1-2017/slides/CG09-TextureFiltering.pdf
 */
fn sample_texture<P: Pixel>(img: &dyn GenericImageView<Pixel = P>, tex: &Vec2) -> (f32, f32, f32, u8, u8) where P: Pixel<Subpixel = u8> {
    // calculate texture dimensions
    let tex_w = (img.width() - 1) as f32;
    let tex_h = (img.height() - 1) as f32;

    // calculate texture coordinate
    let t = Vec2::new(tex.x.rem_euclid(1.0) * tex_w, tex.y.rem_euclid(1.0) * tex_h);

    // sample pixels, offset by 0.5 to "center" pixels
    // NOTE: first vec is in clockwise-order, actual vec is in the order the ref PDF calculations require
    //let p = [(t.x + 1.5, t.y + 1.5), (t.x + 1.5, t.y - 0.5), (t.x - 0.5, t.y - 0.5), (t.x - 0.5, t.y + 1.5)]
    let p = [(t.x - 0.5, t.y - 0.5), (t.x + 1.5, t.y - 0.5), (t.x - 0.5, t.y + 1.5), (t.x + 1.5, t.y + 1.5)]
        .map(|x| {
            // snap the sampling coordinate to pixel-grid
            let snap_x = x.0 as u32;
            let snap_y = x.1 as u32;

            return (snap_x as f32 + 0.5, snap_y as f32 + 0.5, img.get_pixel(snap_x.clamp(0, tex_w as u32), snap_y.clamp(0, tex_h as u32)));
        });
    
    // calculate s and t as in ref PDF
    let dist_s = t.x - p[0].0;
    let dist_t = t.y - p[0].1;

    // return results based on pixel channel count
    match p[0].2.channels().len() {
        2 => {
            let p = p[0].2.to_luma_alpha();
            return (0.0, 0.0, 0.0, p[0], p[1]);
        },
        3 => {
            let p = p
                .map(|x| {
                    let pix = x.2.to_rgb();
                    return Vec3::new(pix[0] as f32 / 255.0, pix[1] as f32 / 255.0, pix[2] as f32 / 255.0);
                });
            let c = (1.0 - dist_t) * (1.0 - dist_s) * p[0]
                + (1.0 - dist_t) * dist_s * p[1]
                + dist_t * (1.0 - dist_s) * p[2]
                + dist_t * dist_s * p[3];
            
            return (c[0], c[1], c[2], 255, 255);
        },
        4 => {
            let p = p
                .map(|x| {
                    let pix = x.2.to_rgba();
                    return Vec4::new(pix[0] as f32 / 255.0, pix[1] as f32 / 255.0, pix[2] as f32 / 255.0, pix[3] as f32);
                });
            let c = (1.0 - dist_t) * (1.0 - dist_s) * p[0]
                + (1.0 - dist_t) * dist_s * p[1]
                + dist_t * (1.0 - dist_s) * p[2]
                + dist_t * dist_s * p[3];
            
            return (c[0], c[1], c[2], 255, p[0].w as u8);
        },
        _ => return (0.0, 0.0, 0.0, 255, 255)
    }
}

impl Raytracer {
    pub fn trace(scene: &Scene, ray: &Ray, n: u8) -> Vec3 {
        // limit recursion
        if n > 4 {
            return RESULT_NULL;
        }

        // get ref to BVH
        let bvh = scene.bvh.as_ref().unwrap();

        // find closest intersection
        let hits = bvh.traverse(&ray, &scene.shapes);
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
                let hit_mat = scene.materials.get(hit_result.mat).unwrap();

                // transparency via alpha texture
                if let Texture::Alpha(ref alpha_texture) = hit_mat.alpha_texture {
                    let c = sample_texture(alpha_texture, &hit_result.tex);
                    if c.3 == 0 {
                        let n_ray = Ray::new(hit_result.pos, ray.direction);
                        return result + Raytracer::trace(scene, &n_ray, n + 1);
                    }
                }
                
                // transparency via diffuse texture
                let mut d_color = hit_mat.diffuse;
                if let Texture::Diffuse(ref diffuse_texture) = hit_mat.diffuse_texture {
                    let c = sample_texture(diffuse_texture, &hit_result.tex);
                    if c.4 == 0 {
                        let n_ray = Ray::new(hit_result.pos, ray.direction);
                        return result + Raytracer::trace(scene, &n_ray, n + 1);
                    }
                    d_color = Vec3::new(c.0, c.1, c.2);
                }
                
                // check if in shadow
                let l_ray = Ray::new(hit_result.pos + hit_result.nrm * EPSILON, -RAYTRACER_LIGHT.normalize());
                let l_hits = bvh.traverse(&l_ray, &scene.shapes);
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
                        let l_hit_mat = scene.materials.get(l_hit_result.mat).unwrap();
                        if let Texture::Alpha(ref alpha_texture) = l_hit_mat.alpha_texture {
                            // transparency via alpha texture
                            let c = sample_texture(alpha_texture, &l_hit_result.tex);
                            if c.3 == 0 {
                                l_shadow = false;
                            }
                        } else if let Texture::Diffuse(ref diffuse_texture) = l_hit_mat.diffuse_texture  {
                            // transparency via diffuse texture
                            let c = sample_texture(diffuse_texture, &l_hit_result.tex);
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
