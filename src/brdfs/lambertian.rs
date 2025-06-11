use crate::brdf::{Brdf, BrdfSample};
use crate::material::Material;
use glam::{Mat3A, Vec3A};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct LambertianBrdf;

impl Brdf for LambertianBrdf {
    fn eval(&self, _view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> Vec3A {
        if normal.dot(light) <= 0.0 {
            return Vec3A::ZERO;
        }

        material.albedo * std::f32::consts::FRAC_1_PI
    }

    fn sample(&self, _view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        let scattered_direction = random_cosine_direction(normal);
        let pdf = normal.dot(scattered_direction).max(0.0) * std::f32::consts::FRAC_1_PI;
        let attenuation = material.albedo;

        BrdfSample {
            direction: scattered_direction,
            attenuation,
            pdf,
        }
    }
}

fn create_orthonormal_basis(normal: Vec3A) -> Mat3A {
    let n = normal;
    let tangent = if n.x.abs() > n.y.abs() {
        Vec3A::new(n.z, 0.0, -n.x) / (n.x * n.x + n.z * n.z).sqrt()
    } else {
        Vec3A::new(0.0, -n.z, n.y) / (n.y * n.y + n.z * n.z).sqrt()
    };
    let bitangent = n.cross(tangent);

    Mat3A::from_cols(tangent, bitangent, n)
}

fn random_cosine_direction(normal: Vec3A) -> Vec3A {
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
