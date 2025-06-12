use crate::material::Material;
use glam::{Mat3A, Vec3A};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct BrdfEval {
    pub f_r: Vec3A,
    pub pdf: f32,
}

#[derive(Debug, Clone)]
pub struct BrdfSample {
    pub attenuation: Vec3A,
    pub direction: Vec3A,
    pub pdf: f32,
}

pub trait Brdf: Send + Sync {
    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> BrdfEval;
    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample;
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn random_cosine_direction(normal: Vec3A) -> Vec3A {
    let r1 = rand::random::<f32>();
    let r2 = rand::random::<f32>();

    let r = r2.sqrt();
    let phi = 2.0 * PI * r1;

    let x = r * phi.cos();
    let y = r * phi.sin();
    let z = (1.0 - r2).max(0.0).sqrt();

    let tbn = create_orthonormal_basis(normal);
    tbn.mul_vec3a(Vec3A::new(x, y, z)).normalize()
}

pub fn create_orthonormal_basis(normal: Vec3A) -> Mat3A {
    let n = normal;
    let tangent = if n.x.abs() > n.y.abs() {
        Vec3A::new(n.z, 0.0, -n.x) / (n.x * n.x + n.z * n.z).sqrt()
    } else {
        Vec3A::new(0.0, -n.z, n.y) / (n.y * n.y + n.z * n.z).sqrt()
    };
    let bitangent = n.cross(tangent);

    Mat3A::from_cols(tangent, bitangent, n)
}
