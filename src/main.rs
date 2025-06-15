mod aabb;
mod brdf;
mod brdfs;
mod camera;
mod hit;
mod material;
mod object;
mod objects;
mod ray;
mod scene;

use crate::{
    brdfs::disney::Disney,
    camera::{Camera, RenderOptions},
    material::Material,
    objects::{r#box::Box, plain::Plain},
    scene::Scene,
};
use glam::{Quat, Vec2, Vec3A};
use std::{fs::File, io::BufWriter, path::Path};

const MATERIAL_WHITE: Material = Material {
    is_emissive: false,
    emission: Vec3A::ZERO,
    albedo: Vec3A::ONE,
    subsurface: 0.0,
    metallic: 0.0,
    specular: 0.0,
    specular_tint: Vec3A::ONE,
    roughness: 1.0,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 1.0,
    clearcoat_gloss: 0.95,
};
const MATERIAL_RED: Material = Material {
    is_emissive: false,
    emission: Vec3A::ZERO,
    albedo: Vec3A::new(1.0, 0.0, 0.0),
    subsurface: 0.0,
    metallic: 0.0,
    specular: 0.0,
    specular_tint: Vec3A::ONE,
    roughness: 1.0,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 1.0,
    clearcoat_gloss: 0.95,
};
const MATERIAL_GREEN: Material = Material {
    is_emissive: false,
    emission: Vec3A::ZERO,
    albedo: Vec3A::new(0.0, 1.0, 0.0),
    subsurface: 0.0,
    metallic: 0.0,
    specular: 0.0,
    specular_tint: Vec3A::ONE,
    roughness: 1.0,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 1.0,
    clearcoat_gloss: 0.95,
};
const MATERIAL_LIGHT: Material = Material {
    is_emissive: true,
    emission: Vec3A::new(10.0, 10.0, 10.0),
    albedo: Vec3A::ZERO,
    subsurface: 0.0,
    metallic: 0.0,
    specular: 0.0,
    specular_tint: Vec3A::ONE,
    roughness: 1.0,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 0.0,
    clearcoat_gloss: 0.0,
};
const MATERIAL_BOX_1: Material = Material {
    is_emissive: false,
    emission: Vec3A::ZERO,
    albedo: Vec3A::new(1.0, 1.0, 1.0),
    subsurface: 0.0,
    metallic: 0.8,
    specular: 0.0,
    specular_tint: Vec3A::ONE,
    roughness: 0.1,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 0.0,
    clearcoat_gloss: 0.0,
};
const MATERIAL_BOX_2: Material = Material {
    is_emissive: false,
    emission: Vec3A::ZERO,
    albedo: Vec3A::new(1.0, 1.0, 1.0),
    subsurface: 0.0,
    metallic: 0.0,
    specular: 0.2,
    specular_tint: Vec3A::ONE,
    roughness: 0.1,
    anisotropic: 0.0,
    sheen: 0.0,
    sheen_tint: Vec3A::ZERO,
    clearcoat: 0.0,
    clearcoat_gloss: 0.0,
};
const BOX_SIZE: f32 = 2.5;
const BOX_THICKNESS: f32 = 0.1;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) * 0.5;
const LIGHT_SIZE: f32 = 0.5;

fn main() {
    let path = Path::new(r"image.png");
    let screen_width = 640;
    let screen_height = 480;

    // Cornell Box
    let mut scene = Scene::new();

    // Walls
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

    // Colored Walls
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

    // Light
    scene.add_object(Plain {
        center: Vec3A::new(0.0, BOX_OFFSET - BOX_THICKNESS * 0.5 - 1e-3, 0.0),
        normal: Vec3A::NEG_Y,
        size: Vec2::new(LIGHT_SIZE, LIGHT_SIZE),
        material: MATERIAL_LIGHT.clone(),
    });

    // Two Boxes
    scene.add_object(Box {
        center: Vec3A::new(-0.35, -BOX_OFFSET + 0.8, -0.35),
        size: Vec3A::new(0.8, 1.6, 0.8),
        rotation: Quat::from_rotation_y(20.0f32.to_radians()),
        material: MATERIAL_BOX_1.clone(),
    });
    scene.add_object(Box {
        center: Vec3A::new(0.45, -BOX_OFFSET + 0.35, 0.35),
        size: Vec3A::new(0.7, 0.7, 0.7),
        rotation: Quat::from_rotation_y(-20.0f32.to_radians()),
        material: MATERIAL_BOX_2.clone(),
    });

    let camera = Camera::look_at(
        Vec3A::new(0.0, 0.0, 3.25),
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 1.0, 0.0),
        60.0,
    );
    let frame_buffer = camera.render(
        &scene,
        &Disney,
        &RenderOptions {
            screen_width,
            screen_height,
            sample_per_pixel: 1024,
            max_ray_bounces: 8,
            exposure: 1.0,
            gamma: 2.2,
        },
    );

    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, screen_width, screen_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&frame_buffer).unwrap();
}
