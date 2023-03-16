pub mod camera;
pub mod intersection;
pub mod material;
pub mod renderer;
pub mod triangle;
pub mod transform;
pub mod utils;
pub mod vertex;

use bvh::{bvh::BVH};
use glam::{Vec3, Vec2};
use image::{ImageBuffer, RgbImage, ImageFormat};
use image::io::Reader as ImageReader;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    camera::Camera,
    material::Material,
    renderer::Raytracer,
    triangle::Triangle,
    vertex::Vertex,
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn load_texture(model_file_name: &str, texture_name: &str) -> Option<RgbImage> {
    if texture_name.is_empty() {
        return None;
    }

    let base_path = PathBuf::from(model_file_name);
    let base_path = base_path
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    let mut file_name = String::from(base_path);
    file_name.push('/');
    file_name.push_str(texture_name);

    return Some(ImageReader::open(file_name)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8());
}

fn load_model(file_name: &str, out_tris: &mut Vec<Triangle>, out_mats: &mut HashMap<String, Material>) {
    println!("loading models and materials...");
    let tobj_load_opts = tobj::LoadOptions {
        triangulate: true,
        ignore_lines: true,
        ignore_points: true,
        single_index: false,
    };
    let (models, materials) = tobj::load_obj(file_name, &tobj_load_opts)
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
        
        out_mats.entry(mat.name.clone()).or_insert(Material {
            ambient: Vec3::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]),
            diffuse: Vec3::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]),
            specular: Vec3::new(mat.specular[0], mat.specular[1], mat.specular[2]),
            shininess: mat.shininess,
            emission: Vec3::new(mat_emission[0], mat_emission[1], mat_emission[2]),
            diffuse_texture: load_texture(file_name, &mat.diffuse_texture),
        });

        for v in vertices.chunks_exact(3) {
            out_tris.push(Triangle {
                vrt: [
                    v[0],
                    v[1],
                    v[2],
                ],
                mat: mat.name.clone(),
                node_idx: 0,
            });
        }
    }
}

fn main() {
    // final render buffer
    let mut render_buf: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

    // scene shapes vector
    let mut scene_shapes: Vec<Triangle> = Vec::new();

    // scene materials map
    let mut scene_materials: HashMap<String, Material> = HashMap::new();

    // load models and materials
    load_model("./res/wirokit.obj", &mut scene_shapes, &mut scene_materials);

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

    // render scene
    for y in 0..scene_cam.viewport_h as u32 {
        for x in 0..scene_cam.viewport_w as u32 {
            let ray = scene_cam.calc_ray(x as f32, y as f32);
            let col = Raytracer::trace(&scene_bvh, &scene_shapes, &scene_materials, &ray, 0) * 255.0;
            let pix = image::Rgb([
                col.x.max(1.0) as u8,
                col.y.max(1.0) as u8,
                col.z.max(1.0) as u8
            ]);
            render_buf.put_pixel(x, y, pix);
        }
    }

    // export render buffer
    render_buf.save_with_format("./render.png", ImageFormat::Png).unwrap();
}
