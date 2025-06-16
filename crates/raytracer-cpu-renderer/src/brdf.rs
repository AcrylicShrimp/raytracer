use glam::Vec3A;
use raytracer_core::material::Material;

#[derive(Debug, Clone)]
pub struct BrdfEval {
    pub f_r: Vec3A,
    pub pdf: f32,
}

impl BrdfEval {
    pub const ZERO: Self = Self {
        f_r: Vec3A::ZERO,
        pdf: 0.0,
    };
}

#[derive(Debug, Clone)]
pub struct BrdfSample {
    pub attenuation: Vec3A,
    pub direction: Vec3A,
    pub pdf: f32,
}

impl BrdfSample {
    pub const ZERO: Self = Self {
        attenuation: Vec3A::ZERO,
        direction: Vec3A::ZERO,
        pdf: 0.0,
    };
}

pub trait Brdf: Send + Sync {
    fn is_delta_surface(&self, material: &Material) -> bool;
    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> BrdfEval;
    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample;
}
