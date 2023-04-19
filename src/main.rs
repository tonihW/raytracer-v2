pub mod camera;
pub mod intersection;
pub mod material;
pub mod renderer;
pub mod scene;
pub mod triangle;
pub mod transform;
pub mod utils;
pub mod vertex;

use bvh::{bvh::BVH};
use clap::{arg, Command};
use glam::{Vec3, Vec2};
use image::{ImageBuffer, RgbImage, ImageFormat, Rgb};
use image::io::Reader as ImageReader;
use material::{Texture, TextureType};
use std::path::PathBuf;
use std::thread::{self, ScopedJoinHandle};

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
        .join(texture_name);
    let file_path = String::from(file_path.to_str().unwrap())
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
            scene.shapes.push(Triangle {
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
            arg!(--model <MODEL>)
                .required(false)
                .default_value("./res/wirokit.obj")
                .value_parser(clap::value_parser!(String))
        )
        .arg(
            arg!(--cam_pos <CAM_POS>)
                .required(false)
                .default_value("0.0 0.0 0.0")
                .value_parser(clap::value_parser!(String))
        )
        .arg(
            arg!(--cam_axis <CAM_AXIS>)
                .required(false)
                .default_value("0.0 1.0 0.0")
                .value_parser(clap::value_parser!(String))
        )
        .arg(
            arg!(--cam_angle <CAM_ANGLE>)
                .required(false)
                .default_value("0.0")
                .value_parser(clap::value_parser!(f32))
        )
        .get_matches();
    let arg_width = args.get_one::<u32>("width").unwrap();
    let arg_height = args.get_one::<u32>("height").unwrap();
    let arg_model = args.get_one::<String>("model").unwrap();
    let arg_cam_pos = args.get_one::<String>("cam_pos")
        .map_or("0.0 0.0 0.0", String::as_str)
        .split(" ")
        .map(|s| s.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    let arg_cam_axis = args.get_one::<String>("cam_axis")
        .map_or("0.0 0.0 0.0", String::as_str)
        .split(" ")
        .map(|s| s.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    let arg_cam_angle = args.get_one::<f32>("cam_angle").unwrap();

    // final render buffer
    let mut render_buf: RgbImage = ImageBuffer::new(*arg_width, *arg_height);

    // init scene
    let mut scene = Scene::new(Camera::from_axis_angle(
        Vec3 { x: arg_cam_pos[0], y: arg_cam_pos[1], z: arg_cam_pos[2] },
        Vec3 { x: arg_cam_axis[0], y: arg_cam_axis[1], z: arg_cam_axis[2] },
        std::f32::consts::PI / 180.0 * *arg_cam_angle,
        *arg_width as f32,
        *arg_height as f32
    ));

    // load models and materials
    load_model(arg_model, &mut scene);

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
