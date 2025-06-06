mod aabb;
mod camera;
mod hit;
mod light;
mod lights;
mod material;
mod object;
mod objects;
mod ray;
mod scene;

use crate::{
    camera::Camera,
    lights::{directional_light::DirectionalLight, spot_light::SpotLight},
    material::Material,
    objects::sphere::Sphere,
    scene::Scene,
};
use glam::Vec3A;
use std::{fs::File, io::BufWriter, path::Path};

fn main() {
    let path = Path::new(r"image.png");
    let screen_width = 640;
    let screen_height = 480;

    let mut scene = Scene::new();
    scene.add_object(Sphere {
        center: Vec3A::new(0.0, 0.0, 5.0),
        radius: 1.0,
        material: Material {
            albedo: Vec3A::new(1.0, 1.0, 0.5),
        },
    });
    scene.add_object(Sphere {
        center: Vec3A::new(1.0, 0.0, 2.0),
        radius: 0.75,
        material: Material {
            albedo: Vec3A::new(0.5, 1.0, 1.0),
        },
    });

    scene.add_light(DirectionalLight {
        color: Vec3A::new(1.0, 1.0, 1.0),
        intensity: 5f32,
        direction: Vec3A::new(-1.0, -1.0, 0.5).normalize(),
    });
    scene.add_light(SpotLight {
        color: Vec3A::new(1.0, 1.0, 1.0),
        intensity: 2f32,
        position: Vec3A::new(0.0, 5.0, 5.0),
    });

    let camera = Camera::look_at(
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 0.0, 1.0),
        Vec3A::new(0.0, 1.0, 0.0),
        90.0,
    );
    let frame_buffer = camera.render(
        &scene,
        screen_width,
        screen_height,
        Vec3A::ZERO,
        1f32,
        2.2f32,
    );

    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, screen_width, screen_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&frame_buffer).unwrap();
}
