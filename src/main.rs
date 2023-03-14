pub mod intersection;
pub mod material;
pub mod utils;
pub mod vertex;
pub mod triangle;
pub mod transform;
pub mod camera;

use bvh::{bvh::BVH, ray::Ray};
use glam::{Vec3, Vec2, Quat};
use image::{ImageBuffer, RgbImage, ImageFormat};
use intersection::Intersection;
use triangle::Triangle;
use transform::Transform;
use camera::Camera;

use crate::{vertex::Vertex, material::Material, utils::EPSILON};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const LIGHT: Vec3 = Vec3::new(0.5, -1.0, 0.4);
const AMBIENT: Vec3 = Vec3::new(0.4, 0.4, 0.6);

fn main() {
    // final render buffer
    let mut render_buf: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

    // scene shapes vector
    let mut scene_shapes: Vec<Triangle> = Vec::new();

    // load models and materials
    println!("loading models and materials...");
    let tobj_load_opts = tobj::LoadOptions {
        triangulate: true,
        ignore_lines: false,
        ignore_points: false,
        single_index: false,
    };
    let (models, materials) = tobj::load_obj("./res/wirokit.obj", &tobj_load_opts)
        .expect("  failed to load target OBJ file");
    let materials = materials.expect("  failed to load target MTL file");

    for m in &models {
        println!("  model.name = \"{}\"", m.name);
        println!("  model.mesh.material_id = {:?}", m.mesh.material_id);
        println!("  model.indice_count = {}", m.mesh.indices.len());
        println!("  model.normal_indice_count = {}", m.mesh.normal_indices.len());
        println!("  model.texcoord_indice_count = {}", m.mesh.texcoord_indices.len());
        println!("  model.face_count = {}", m.mesh.indices.len() / 3);

        let mat = &materials[m.mesh.material_id.unwrap()];
        println!("  material.name = {}", mat.name);
        println!("  material.unknown_param_count = {}", mat.unknown_param.len());
        for (k, v) in &mat.unknown_param {
            println!("    unknown_param[{}] = {}", k, v);
        }
        let mat_emission = &mat.unknown_param["Ke"];
        let mat_emission = mat_emission
            .split(" ")
            .map(|s| s.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        println!("  material.emission = {} {} {}", mat_emission[0], mat_emission[1], mat_emission[2]);

        let mut vertices: Vec<Vertex> = Vec::new();
        for i in 0..m.mesh.indices.len() {
            let p_offset = (m.mesh.indices[i] * 3) as usize;
            let n_offset = (m.mesh.normal_indices[i] * 3) as usize;
            let t_offset = (m.mesh.texcoord_indices[i] * 2) as usize;

            let pos = Vec3::new(m.mesh.positions[p_offset + 0], m.mesh.positions[p_offset + 1], m.mesh.positions[p_offset + 2]);
            let nrm = Vec3::new(m.mesh.normals[n_offset + 0], m.mesh.normals[n_offset + 1], m.mesh.normals[n_offset + 2]);
            let tex = Vec2::new(m.mesh.texcoords[t_offset + 0], m.mesh.texcoords[t_offset + 1]);

            vertices.push(Vertex {
                pos,
                nrm,
                tex,
            });
        }

        for v in vertices.chunks_exact(3) {
            scene_shapes.push(Triangle {
                vrt: [
                    v[0],
                    v[1],
                    v[2],
                ],
                mat: Material {
                    ambient: Vec3::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]),
                    diffuse: Vec3::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]),
                    specular: Vec3::new(mat.specular[0], mat.specular[1], mat.specular[2]),
                    shininess: mat.shininess,
                    emission: Vec3::new(mat_emission[0], mat_emission[1], mat_emission[2]),
                },
                node_idx: 0,
            });
        }
    }

    // construct scene
    println!("constructing scene, shape_count: {} ...", scene_shapes.len());
    let scene_bvh = BVH::build(&mut scene_shapes);
    let scene_cam = Camera::from_axis_angle(
        Vec3 { x: 4.0, y: 0.5, z: 0.0 },
        Vec3 { x: 0.0, y: 1.0, z: 0.0 },
        -std::f32::consts::PI / 180.0 * 90.0,
        WIDTH as f32,
        HEIGHT as f32
    );

    // test raytrace
    for y in 0..scene_cam.viewport_h as u32 {
        for x in 0..scene_cam.viewport_w as u32 {
            let ray = scene_cam.calc_ray(x as f32, y as f32);
            let hits = scene_bvh.traverse(&ray, &scene_shapes);
            let mut dist = f32::MAX;
            let mut isect: Option<Intersection> = None;
            for hit in hits {
                match hit.intersect(&ray) {
                    Some(result) => {
                        if result.t < dist {
                            dist = result.t;
                            isect = Some(result);
                        }
                    },
                    None => (),
                }
            }
            match isect {
                Some(result) => {
                    let mut mat_col = result.mat.diffuse;

                    // mutable color for final pixel
                    let mut color = AMBIENT * mat_col;

                    // check if in shadow
                    let l_ray = Ray::new(result.pos + result.nrm * EPSILON, -LIGHT.normalize());
                    let l_hits = scene_bvh.traverse(&l_ray, &scene_shapes);
                    let mut l_dist = f32::MAX;
                    let mut l_shadow = false;
                    for l_hit in l_hits {
                        match l_hit.intersect(&l_ray) {
                            Some(l_result) => {
                                if l_result.t < l_dist {
                                    l_dist = l_result.t;
                                    l_shadow = true;
                                }
                            },
                            None => (),
                        }
                    }
                    
                    if !l_shadow {
                        let n_dot_l = result.nrm.dot(-LIGHT.normalize());
                        color += n_dot_l * mat_col;
                    }

                    color *= 255.0;

                    let pix = image::Rgb([
                        color.x.max(1.0) as u8,
                        color.y.max(1.0) as u8,
                        color.z.max(1.0) as u8
                    ]);
                    render_buf.put_pixel(x, y, pix);
                },
                None => {
                    let r_dot_l = ray.direction.dot(-LIGHT.normalize());
                    let color = (AMBIENT + (r_dot_l * AMBIENT)) * 255.0;
                    let pix = image::Rgb([
                        color.x.max(1.0) as u8,
                        color.y.max(1.0) as u8,
                        color.z.max(1.0) as u8
                    ]);
                    render_buf.put_pixel(x, y, pix);
                },
            }
        }
    }

    // export render buffer
    render_buf.save_with_format("./render.png", ImageFormat::Png).unwrap();
}
