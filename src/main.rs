pub mod camera;
pub mod intersection;
pub mod material;
pub mod light;
pub mod renderer;
pub mod scene;
pub mod triangle;
pub mod transform;
pub mod utils;
pub mod vertex;

use bvh::aabb::Bounded;
use bvh::{bvh::BVH};
use clap::{arg, Command};
use glam::{Vec3, Vec2};
use image::{ImageBuffer, RgbImage, ImageFormat, Rgb};
use image::io::Reader as ImageReader;
use material::{Texture, TextureType};
use std::fs;
use std::path::PathBuf;
use std::thread::{self, ScopedJoinHandle};

use crate::light::{DirLight, PointLight};
use crate::scene::Scene;
use crate::{
    camera::Camera,
    material::Material,
    renderer::Raytracer,
    triangle::Triangle,
    vertex::Vertex,
};

fn load_texture(model_file_name: &str, texture_name: &str, texture_type: TextureType) -> Texture {
    // return None if nothing to load
    if texture_name.is_empty() {
        return Texture::None;
    }

    // attempt to load file into memory
    let file_path = PathBuf::from(model_file_name)
        .parent()
        .unwrap()
        .join(texture_name)
        .to_str()
        .unwrap()
        .replace('\\', "/");
    let image = ImageReader::open(file_path)
        .unwrap()
        .decode()
        .unwrap();

    // return type based on condition
    match texture_type {
        TextureType::Diffuse => {
            println!("loading diffuse texture ...");

            match image.color().has_alpha() {
                true => return Texture::Diffuse(image.to_rgba8()),
                false => {
                    let mut image_rgb = image.to_rgba8();
                    image_rgb.pixels_mut().for_each(|p| p[3] = 255);
                    return Texture::Diffuse(image_rgb);
                },
            }
        },
        TextureType::Alpha => {
            println!("loading alpha texture ...");

            return Texture::Alpha(image.to_luma_alpha8());
        },
        TextureType::None => {
            println!("loading none texture ...");

            return Texture::None;
        },
    }
}

fn load_model(file_name: &str, scene: &mut Scene) {
    println!("loading models and materials...");
    let tobj_load_opts = tobj::LoadOptions {
        triangulate: true,
        ignore_lines: true,
        ignore_points: true,
        single_index: false,
    };
    let (models, materials) = tobj::load_obj(file_name, &tobj_load_opts)
        .expect("  failed to load target OBJ file");
    let mut materials = materials.expect("  failed to load target MTL file");

    for m in &models {
        println!("  model.name = \"{}\"", m.name);
        println!("  model.mesh.material_id = {:?}", m.mesh.material_id);
        println!("  model.indice_count = {}", m.mesh.indices.len());
        println!("  model.normal_indice_count = {}", m.mesh.normal_indices.len());
        println!("  model.texcoord_indice_count = {}", m.mesh.texcoord_indices.len());
        println!("  model.face_count = {}", m.mesh.indices.len() / 3);

        let mat = match m.mesh.material_id {
            Some(material_id) => &materials[material_id],
            None => {
                if !materials.is_empty() {
                    &materials[0]
                } else {
                    materials.push(tobj::Material::default());

                    &materials[0]
                }
            }
        };

        println!("  material.name = {}", mat.name);
        println!("  material.unknown_param_count = {}", mat.unknown_param.len());
        for (k, v) in &mat.unknown_param {
            println!("    unknown_param[{}] = {}", k, v);
        }
        let mat_emission = mat.unknown_param.get("Ke")
            .map_or("0 0 0", String::as_str)    
            .split(" ")
            .map(|s| s.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        println!("  material.emission = {} {} {}", mat_emission[0], mat_emission[1], mat_emission[2]);
        println!("  material.diffuse_texture = {}", &mat.diffuse_texture);
        println!("  material.alpha_texture = {}", &mat.dissolve_texture);

        if !scene.materials.contains_key(&mat.name) {
            scene.materials.insert(mat.name.clone(), Material {
                ambient: Vec3::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]),
                diffuse: Vec3::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]),
                specular: Vec3::new(mat.specular[0], mat.specular[1], mat.specular[2]),
                shininess: mat.shininess,
                emission: Vec3::new(mat_emission[0], mat_emission[1], mat_emission[2]),
                diffuse_texture: load_texture(file_name, &mat.diffuse_texture, TextureType::Diffuse),
                alpha_texture: load_texture(file_name, &mat.dissolve_texture, TextureType::Alpha),
            });
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        for i in 0..m.mesh.indices.len() {
            let p_offset = (m.mesh.indices[i] * 3) as usize;
            let pos = Vec3::new(m.mesh.positions[p_offset + 0], m.mesh.positions[p_offset + 1], m.mesh.positions[p_offset + 2]);

            let mut nrm = Vec3::new(0.0, 0.0, 0.0);
            if m.mesh.normal_indices.len() > 0 {
                let n_offset = (m.mesh.normal_indices[i] * 3) as usize;
                nrm = Vec3::new(m.mesh.normals[n_offset + 0], m.mesh.normals[n_offset + 1], m.mesh.normals[n_offset + 2]);
            }

            let mut tex = Vec2::new(0.0, 0.0);
            if m.mesh.texcoord_indices.len() > 0 {
                let t_offset = (m.mesh.texcoord_indices[i] * 2) as usize;
                tex = Vec2::new(m.mesh.texcoords[t_offset + 0], m.mesh.texcoords[t_offset + 1]);
            }

            vertices.push(Vertex {
                pos,
                nrm,
                tex,
            });
        }
        
        for v in vertices.chunks_exact_mut(3) {
            // calculate normals if not set
            if v[0].nrm.length() == 0.0 && v[1].nrm.length() == 0.0 && v[2].nrm.length() == 0.0 {
                let edge_a = v[0].pos - v[1].pos;
                let edge_b = v[0].pos - v[2].pos;
                let nrm = edge_a.cross(edge_b).normalize();
                v[0].nrm = nrm;
                v[1].nrm = nrm;
                v[2].nrm = nrm;
            }

            let t = Triangle {
                vrt: [
                    v[0],
                    v[1],
                    v[2],
                ],
                mat: mat.name.clone(),
                node_idx: 0,
            };

            // validate triangle, discard invalid triangles
            if Bounded::aabb(&t).surface_area() > 0.0 {
                scene.shapes.push(t);
            }
        }
    }
}

fn main() {
    // parse args
    let args = Command::new("pathtracer")
        .version("0.1.0")
        .author("tonihW")
        .about("Simple 3D renderer based on raytracing")
        .arg(
            arg!(--width <WIDTH>)
                .required(false)
                .default_value("512")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            arg!(--height <HEIGHT>)
                .required(false)
                .default_value("512")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            arg!(--scene <SCENE>)
                .required(false)
                .default_value("./res/wirokit.json")
                .value_parser(clap::value_parser!(String))
        )
        .get_matches();
    let arg_width = args.get_one::<u32>("width").unwrap();
    let arg_height = args.get_one::<u32>("height").unwrap();
    let arg_scene = args.get_one::<String>("scene").unwrap();

    // final render buffer
    let mut render_buf: RgbImage = ImageBuffer::new(*arg_width, *arg_height);

    // load scene file
    let scene_json_file = fs::File::open(arg_scene)
        .expect("Failed to load target scene JSON file");
    let scene_json: serde_json::Value = serde_json::from_reader(scene_json_file)
        .expect("Failed to parse target scene JSON file");

    // load camera params
    let camera_json = scene_json.get("camera")
        .expect("camera is a mandatory field for a scene JSON file");
    let camera_pos: Vec<f32> = camera_json.get("position")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect();
    let camera_axis: Vec<f32> = camera_json.get("rot_axis")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect();
    let camera_angle = camera_json.get("rot_angle")
        .unwrap()
        .as_f64()
        .unwrap();

    // init scene
    let mut scene = Scene::new(Camera::from_axis_angle(
        Vec3 { x: camera_pos[0] , y: camera_pos[1], z: camera_pos[2] },
        Vec3 { x: camera_axis[0], y: camera_axis[1], z: camera_axis[2] },
        std::f32::consts::PI / 180.0 * camera_angle as f32,
        *arg_width as f32,
        *arg_height as f32
    ));

    // load models and materials
    for model in scene_json["models"].as_array().unwrap() {
        load_model(model.as_str().unwrap(), &mut scene);
    }

    // load lights
    for light in scene_json["lights"].as_array().unwrap() {
        let light_type = light.get("type")
            .unwrap()
            .as_str()
            .unwrap();

        println!("loading light of type \"{light_type}\"");
        match light_type {
            "AmbientLight" => {
                let light_emission: Vec<f32> = light.get("emission")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32)
                    .collect();

                scene.ambient = Vec3::new(light_emission[0], light_emission[1], light_emission[2]);
            },
            "DirLight" => {
                let light_direction: Vec<f32> = light.get("direction")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32)
                    .collect();
                let light_emission: Vec<f32> = light.get("emission")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32)
                    .collect();

                scene.lights.push(Box::new(DirLight {
                    direction: Vec3::new(light_direction[0], light_direction[1], light_direction[2]),
                    emission: Vec3::new(light_emission[0], light_emission[1], light_emission[2]),
                }));
            },
            "PointLight" => {
                let light_position: Vec<f32> = light.get("position")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32)
                    .collect();
                let light_emission: Vec<f32> = light.get("emission")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32)
                    .collect();
                let light_c = light.get("c")
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let light_l = light.get("l")
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let light_q = light.get("q")
                    .unwrap()
                    .as_f64()
                    .unwrap();

                scene.lights.push(Box::new(PointLight {
                    position: Vec3::new(light_position[0], light_position[1], light_position[2]),
                    emission: Vec3::new(light_emission[0], light_emission[1], light_emission[2]),
                    c: light_c as f32,
                    l: light_l as f32,
                    q: light_q as f32,
                }));
            },
            _ => ()
        }
    }

    // construct scene
    println!("constructing scene, shape_count: {} ...", scene.shapes.len());
    scene.bvh = Some(BVH::build(&mut scene.shapes));

    // determine multithreading params
    let cpu_count = thread::available_parallelism()
        .unwrap()
        .get();
    let task_w = scene.camera.viewport_w as usize / cpu_count;
    let task_h = scene.camera.viewport_h as usize / cpu_count;
    println!("cpu_count: {}, task_w: {}, task_h: {}", cpu_count, task_w, task_h);

    // execute rendering as split tasks across multiple threads
    thread::scope(|s| {
        let scn = &scene;

        // divide screen into rectangles as individual rendering tasks
        let mut threads: Vec<ScopedJoinHandle<Vec<(u32, u32, Rgb<u8>)>>> = Vec::new();
        for j in 0..cpu_count {
            let y = j * task_h;
            let h = y + task_h;

            for i in 0..cpu_count {
                let x = i * task_w;
                let w = x + task_w;

                threads.push(s.spawn(move || {
                    let mut buf: Vec<(u32, u32, Rgb<u8>)> = Vec::new();

                    for yy in y..h {
                        for xx in x..w {
                            let ray = &scn.camera.calc_ray(xx as f32, yy as f32);
                            let col = Raytracer::trace(scn, &ray, 0) * 255.0;
                            let pix = image::Rgb([
                                col.x.max(1.0) as u8,
                                col.y.max(1.0) as u8,
                                col.z.max(1.0) as u8
                            ]);
                            buf.push((xx as u32, yy as u32, pix));
                        }
                    }

                    return buf;
                }));
            }
        }

        // wait for rendering tasks to complete
        for handle in threads {
            let buf = handle.join().unwrap();

            for pix in buf {
                render_buf.put_pixel(pix.0, pix.1, pix.2);
            }
        }
    });

    // export render buffer
    render_buf.save_with_format("./render.png", ImageFormat::Png).unwrap();
}
