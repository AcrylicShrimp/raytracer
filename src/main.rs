mod aabb;
mod brdf;
mod brdfs;
mod camera;
mod hit;
mod light;
mod lights;
mod material;
mod object;
mod objects;
mod ray;
mod scene;

use crate::{camera::Camera, material::Material, objects::r#box::Box, scene::Scene};
use glam::{Quat, Vec3A};
use std::{fs::File, io::BufWriter, path::Path};

const MATERIAL_WHITE: Material = Material {
    albedo: Vec3A::ONE,
    metallic: 0.5,
    roughness: 1.0,
    is_reflective: false,
};
const MATERIAL_RED: Material = Material {
    albedo: Vec3A::new(1.0, 0.0, 0.0),
    metallic: 0.5,
    roughness: 1.0,
    is_reflective: false,
};
const MATERIAL_GREEN: Material = Material {
    albedo: Vec3A::new(0.0, 1.0, 0.0),
    metallic: 0.5,
    roughness: 1.0,
    is_reflective: false,
};
const MATERIAL_LIGHT: Material = Material {
    albedo: Vec3A::new(1.0, 0.8, 0.6),
    metallic: 0.5,
    roughness: 1.0,
    is_reflective: false,
};
const BOX_SIZE: f32 = 2.5;
const BOX_THICKNESS: f32 = 0.1;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) * 0.5;
const LIGHT_SIZE: f32 = 0.5;
const LIGHT_THICKNESS: f32 = 0.05;

fn main() {
    let path = Path::new(r"image.png");
    let screen_width = 640;
    let screen_height = 480;

    // Cornell Box
    let mut scene = Scene::new();
    scene.add_object(Box {
        center: Vec3A::new(0.0, -BOX_OFFSET, 0.0),
        size: Vec3A::new(BOX_SIZE, BOX_THICKNESS, BOX_SIZE),
        rotation: Quat::IDENTITY,
        material: MATERIAL_WHITE.clone(),
    });
    scene.add_object(Box {
        center: Vec3A::new(0.0, 0.0, -BOX_OFFSET),
        size: Vec3A::new(BOX_SIZE, BOX_SIZE, BOX_THICKNESS),
        rotation: Quat::IDENTITY,
        material: MATERIAL_WHITE.clone(),
    });
    scene.add_object(Box {
        center: Vec3A::new(0.0, BOX_OFFSET, 0.0),
        size: Vec3A::new(BOX_SIZE, BOX_THICKNESS, BOX_SIZE),
        rotation: Quat::IDENTITY,
        material: MATERIAL_WHITE.clone(),
    });

    scene.add_object(Box {
        center: Vec3A::new(-BOX_OFFSET, 0.0, 0.0),
        size: Vec3A::new(BOX_THICKNESS, BOX_SIZE, BOX_SIZE),
        rotation: Quat::IDENTITY,
        material: MATERIAL_RED.clone(),
    });
    scene.add_object(Box {
        center: Vec3A::new(BOX_OFFSET, 0.0, 0.0),
        size: Vec3A::new(BOX_THICKNESS, BOX_SIZE, BOX_SIZE),
        rotation: Quat::IDENTITY,
        material: MATERIAL_GREEN.clone(),
    });

    scene.add_object(Box {
        center: Vec3A::new(0.0, BOX_OFFSET - LIGHT_THICKNESS * 0.5, 0.0),
        size: Vec3A::new(LIGHT_SIZE, LIGHT_SIZE, LIGHT_THICKNESS),
        rotation: Quat::IDENTITY,
        material: MATERIAL_LIGHT.clone(),
    });

    let camera = Camera::look_at(
        Vec3A::new(0.0, 0.0, 3.25),
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 1.0, 0.0),
        120.0,
    );
    let frame_buffer = camera.render(
        &scene,
        screen_width,
        screen_height,
        Vec3A::ONE,
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
