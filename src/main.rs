pub mod intersection;
pub mod vertex;
pub mod triangle;
pub mod transform;
pub mod camera;

use transform::Transform;
use camera::Camera;
use glam::{Vec3, Quat};

fn main() {
    // load models and materials
    let (models, materials) = tobj::load_obj("./res/wirokit.obj", &tobj::LoadOptions::default())
        .expect("Failed to load target OBJ file");
    let materials = materials.expect("Failed to load target MTL file");

    let camera = Camera::from_axis_angle(Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }, 0.0, 1280.0, 720.0);
    let ray = camera.calc_ray(10.0, 30.0);

    println!("{:?}", ray.1);
}
