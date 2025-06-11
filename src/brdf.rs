use crate::material::Material;
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct BrdfSample {
    pub attenuation: Vec3A,
    pub direction: Vec3A,
    pub pdf: f32,
}

pub trait Brdf: Send + Sync {
    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> Vec3A;
    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample;
}
