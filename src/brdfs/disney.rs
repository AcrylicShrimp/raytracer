use crate::{
    brdf::{Brdf, BrdfSample},
    material::Material,
};
use glam::Vec3A;

pub struct Disney;

impl Brdf for Disney {
    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> Vec3A {
        let half = (view + light).normalize();
        let n_dot_v = normal.dot(view);
        let n_dot_l = normal.dot(light);
        let l_dot_h = light.dot(half);

        let dielectric_f0 = Vec3A::splat(material.specular * 0.08);
        let metallic_f0 = material.albedo;
        let f0 = dielectric_f0.lerp(metallic_f0, material.metallic);

        let diffuse_term = diffuse_term(
            n_dot_v,
            n_dot_l,
            l_dot_h,
            material.roughness,
            material.albedo,
        );
        let specular_term = specular_term(n_dot_v, n_dot_l, l_dot_h, material.roughness, f0);
        let clearcoat_term = clearcoat_term(n_dot_v, n_dot_l, l_dot_h, material.clearcoat_gloss);

        todo!()
    }

    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        todo!()
    }
}

fn distribution_term_specular(l_dot_h: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let denom_core = l_dot_h * l_dot_h * (alpha2 - 1.0) + 1.0;
    let denom = std::f32::consts::PI * denom_core * denom_core;

    alpha2 / denom.max(1e-3)
}

fn distribution_term_clearcoat(l_dot_h: f32, gloss: f32) -> f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    let alpha = lerp(0.2, 0.001, gloss);
    let alpha2 = alpha * alpha;

    let denom_core = l_dot_h * l_dot_h * (alpha2 - 1.0) + 1.0;
    let c = if (alpha2 - 1.0).abs() < 1e-3 {
        1.0 / std::f32::consts::PI
    } else {
        (alpha2 - 1.0) / (std::f32::consts::PI * alpha2.ln())
    };

    c / denom_core.max(1e-3)
}

fn fresnel_term(l_dot_h: f32, f0: Vec3A) -> Vec3A {
    f0 + (1.0 - f0) * (1.0 - l_dot_h).powf(5.0)
}

fn geometry_term(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    fn g1(n_dot_x: f32, k: f32) -> f32 {
        n_dot_x / (n_dot_x * (1.0 - k) + k).max(1e-3)
    }

    let k = (roughness + 1.0).powf(2.0) / 8.0;

    g1(n_dot_v, k) * g1(n_dot_l, k)
}

fn diffuse_term(
    n_dot_v: f32,
    n_dot_l: f32,
    l_dot_h: f32,
    roughness: f32,
    base_color: Vec3A,
) -> Vec3A {
    let fd90 = 0.5 + 2.0 * roughness * l_dot_h * l_dot_h;
    let fdv = 1.0 + (fd90 - 1.0) * (1.0 - n_dot_v).powf(5.0);
    let fdl = 1.0 + (fd90 - 1.0) * (1.0 - n_dot_l).powf(5.0);

    base_color * (fdv * fdl / std::f32::consts::PI).max(0.0)
}

fn specular_term(n_dot_v: f32, n_dot_l: f32, l_dot_h: f32, roughness: f32, f0: Vec3A) -> Vec3A {
    distribution_term_specular(l_dot_h, roughness)
        * fresnel_term(l_dot_h, f0)
        * geometry_term(n_dot_v, n_dot_l, roughness)
        / (4.0 * n_dot_v * n_dot_l)
}

fn clearcoat_term(n_dot_v: f32, n_dot_l: f32, l_dot_h: f32, gloss: f32) -> Vec3A {
    distribution_term_clearcoat(l_dot_h, gloss)
        * fresnel_term(l_dot_h, Vec3A::splat(0.04))
        * geometry_term(n_dot_v, n_dot_l, 0.25)
        / (4.0 * n_dot_v * n_dot_l)
}
