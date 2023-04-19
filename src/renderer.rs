use bvh::{ray::Ray};
use glam::{Vec3, Vec2};
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

fn sample_texture<P: Pixel>(img: &dyn GenericImageView<Pixel = P>, tex: &Vec2) -> (f32, f32, f32, u8, u8) where P: Pixel<Subpixel = u8> {
    // get pixel sample at texture coordinate, use wrapping  sampling mode
    let img_w = img.width() - 1;
    let img_h = img.height() - 1;
    let pix_x = (tex.x.rem_euclid(1.0) * img_w as f32) as u32;
    let pix_y = (tex.y.rem_euclid(1.0) * img_h as f32) as u32;
    let pix_c = img.get_pixel(pix_x, pix_y);

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
