pub mod intersection;
pub mod material;
pub mod utils;
pub mod vertex;
pub mod triangle;
pub mod transform;
pub mod camera;

use bvh::bvh::BVH;
use glam::{Vec3, Vec2, Quat};
use triangle::Triangle;
use transform::Transform;
use camera::Camera;

use crate::{vertex::Vertex, material::Material};

fn main() {
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
        .expect("Failed to load target OBJ file");
    let materials = materials.expect("Failed to load target MTL file");

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

    // let camera = Camera::from_axis_angle(Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }, 0.0, 1280.0, 720.0);
    // let ray = camera.calc_ray(10.0, 30.0);

    // println!("{:?}", ray.1);
}
