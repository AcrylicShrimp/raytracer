use crate::brdf::{Brdf, BrdfEval, BrdfSample, random_cosine_direction};
use crate::material::Material;
use glam::Vec3A;
use std::f32::consts::FRAC_1_PI;

#[derive(Debug, Clone, Copy)]
pub struct LambertianBrdf;

impl Brdf for LambertianBrdf {
    fn is_delta_surface(&self, _material: &Material) -> bool {
        false
    }

    fn eval(&self, _view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> BrdfEval {
        if normal.dot(light) <= 0.0 {
            return BrdfEval::ZERO;
        }

        BrdfEval {
            f_r: material.albedo * FRAC_1_PI,
            pdf: normal.dot(light).max(0.0) * FRAC_1_PI,
        }
    }

    fn sample(&self, _view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        let light = random_cosine_direction(normal);
        let pdf = normal.dot(light).max(0.0) * FRAC_1_PI;
        let attenuation = material.albedo;

        BrdfSample {
            direction: light,
            attenuation,
            pdf,
        }
    }
}
