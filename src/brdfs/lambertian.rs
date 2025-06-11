use crate::brdf::{Brdf, BrdfSample, random_cosine_direction};
use crate::material::Material;
use glam::Vec3A;
use std::f32::consts::{FRAC_1_PI, PI};

#[derive(Debug, Clone)]
pub struct LambertianBrdf;

impl Brdf for LambertianBrdf {
    fn eval(&self, _view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> Vec3A {
        if normal.dot(light) <= 0.0 {
            return Vec3A::ZERO;
        }

        material.albedo * FRAC_1_PI
    }

    fn sample(&self, _view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        let scattered_direction = random_cosine_direction(normal);
        let pdf = normal.dot(scattered_direction).max(0.0) * FRAC_1_PI;
        let attenuation = material.albedo;

        BrdfSample {
            direction: scattered_direction,
            attenuation,
            pdf,
        }
    }
}
