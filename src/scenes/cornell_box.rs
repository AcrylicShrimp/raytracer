use glam::{Quat, Vec2, Vec3A};
use raytracer_core::{camera::Camera, material::Material, scene::Scene};
use raytracer_primitives::{Box, Plain};

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

pub fn create_cornell_box() -> (Scene, Camera) {
    let mut scene = Scene::new("Cornell Box");

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

    (scene, camera)
}
