use crate::{
    brdf::{Brdf, BrdfEval, BrdfSample, create_orthonormal_basis, lerp, random_cosine_direction},
    material::Material,
};
use glam::Vec3A;
use std::f32::consts::{FRAC_1_PI, PI};

pub struct Disney;

impl Disney {
    fn compute_lobe_weights(material: &Material) -> (f32, f32, f32) {
        // the clearcoat lobe has maximum 25% chance of being sampled
        let p_clearcoat_lobe = 0.25 * material.clearcoat;
        let p_base_lobe = 1.0 - p_clearcoat_lobe;

        let metallic_weight = material.metallic;
        let dielectric_weight = 1.0 - material.metallic;

        let dielectric_specular_weight = material.specular;
        let dielectric_diffuse_weight = 1.0 - material.specular;

        // specular = metallic specular or dielectric specular
        let specular_weight = metallic_weight + dielectric_weight * dielectric_specular_weight;
        // diffuse = **ONLY** dielectric diffuse
        let diffuse_weight = dielectric_weight * dielectric_diffuse_weight;

        let p_specular_lobe = p_base_lobe * specular_weight;
        let p_diffuse_lobe = p_base_lobe * diffuse_weight;

        (p_clearcoat_lobe, p_specular_lobe, p_diffuse_lobe)
    }
}

impl Brdf for Disney {
    fn is_delta_surface(&self, material: &Material) -> bool {
        material.roughness < 1e-5 && (1.0 - material.metallic).abs() < 1e-5
    }

    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> BrdfEval {
        let half = (view + light).normalize();

        let n_dot_h = normal.dot(half);
        let n_dot_l = normal.dot(light);
        let n_dot_v = normal.dot(view);
        let v_dot_h = view.dot(half);
        let l_dot_h = light.dot(half);

        if n_dot_l < 1e-5 {
            return BrdfEval::ZERO;
        }

        let pdf_clearcoat: f32 = ggx_pdf_clearcoat(n_dot_h, v_dot_h, material.clearcoat_gloss);
        let pdf_specular = ggx_pdf_specular(n_dot_h, v_dot_h, material.roughness);
        let pdf_diffuse = n_dot_l.max(0.0) * FRAC_1_PI;

        let dielectric_f0 = Vec3A::splat(material.specular * 0.08);
        let metallic_f0 = material.albedo;
        let f0 = dielectric_f0.lerp(metallic_f0, material.metallic);

        let clearcoat_term =
            clearcoat_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.clearcoat_gloss);
        let specular_term =
            specular_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.roughness, f0);
        let diffuse_term = diffuse_term(
            n_dot_v,
            n_dot_l,
            l_dot_h,
            material.roughness,
            material.albedo,
        );

        let (p_clearcoat_lobe, p_specular_lobe, p_diffuse_lobe) =
            Self::compute_lobe_weights(material);

        let diffuse_weight = (1.0 - material.metallic) * (1.0 - material.specular);
        let f_r =
            clearcoat_term + (1.0 - diffuse_weight) * specular_term + diffuse_weight * diffuse_term;
        let pdf = p_clearcoat_lobe * pdf_clearcoat
            + p_specular_lobe * pdf_specular
            + p_diffuse_lobe * pdf_diffuse;

        if pdf < 1e-5 {
            return BrdfEval::ZERO;
        }

        BrdfEval { f_r, pdf }
    }

    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        let (p_clearcoat_lobe, p_specular_lobe, _p_diffuse_lobe) =
            Self::compute_lobe_weights(material);
        let dice = rand::random::<f32>();

        let light = if dice < p_clearcoat_lobe {
            let half = gtr1_importance_sample(normal, material.clearcoat_gloss);
            (-view).reflect(half)
        } else if dice < p_clearcoat_lobe + p_specular_lobe {
            let half = gtr2_importance_sample(normal, material.roughness);
            (-view).reflect(half)
        } else {
            random_cosine_direction(normal)
        };
        let BrdfEval { f_r, pdf } = self.eval(view, normal, light, material);

        if pdf < 1e-5 {
            return BrdfSample::ZERO;
        }

        let n_dot_l = normal.dot(light).max(0.0);
        let attenuation = f_r * n_dot_l / pdf;

        BrdfSample {
            direction: light,
            attenuation,
            pdf,
        }
    }
}

fn distribution_term_specular(n_dot_h: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let denom_core = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let denom = std::f32::consts::PI * denom_core * denom_core;

    alpha2 / denom.max(1e-5)
}

fn distribution_term_clearcoat(n_dot_h: f32, gloss: f32) -> f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    let alpha = lerp(0.1, 0.001, gloss);
    let alpha2 = alpha * alpha;

    let denom_core = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let c = if (alpha2 - 1.0).abs() < 1e-5 {
        1.0 / std::f32::consts::PI
    } else {
        (alpha2 - 1.0) / (std::f32::consts::PI * alpha2.ln())
    };

    c / denom_core.max(1e-5)
}

fn fresnel_term(l_dot_h: f32, f0: Vec3A) -> Vec3A {
    f0 + (Vec3A::ONE - f0) * (1.0 - l_dot_h).powf(5.0)
}

fn geometry_term(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    fn g1(n_dot_x: f32, k: f32) -> f32 {
        n_dot_x / (n_dot_x * (1.0 - k) + k).max(1e-5)
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

    base_color * (fdv * fdl * FRAC_1_PI).max(0.0)
}

fn specular_term(
    n_dot_h: f32,
    n_dot_v: f32,
    n_dot_l: f32,
    l_dot_h: f32,
    roughness: f32,
    f0: Vec3A,
) -> Vec3A {
    let denom = (4.0 * n_dot_v * n_dot_l).max(1e-5);

    distribution_term_specular(n_dot_h, roughness)
        * fresnel_term(l_dot_h, f0)
        * geometry_term(n_dot_v, n_dot_l, roughness)
        / denom
}

fn clearcoat_term(n_dot_h: f32, n_dot_v: f32, n_dot_l: f32, l_dot_h: f32, gloss: f32) -> Vec3A {
    let denom = (4.0 * n_dot_v * n_dot_l).max(1e-5);

    distribution_term_clearcoat(n_dot_h, gloss)
        * fresnel_term(l_dot_h, Vec3A::splat(0.04))
        * geometry_term(n_dot_v, n_dot_l, 0.25)
        / denom
}

fn gtr2_importance_sample(normal: Vec3A, roughness: f32) -> Vec3A {
    let r1 = rand::random::<f32>();
    let r2 = rand::random::<f32>();

    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let cos_theta = ((1.0 - r1) / (r1 * (alpha2 - 1.0) + 1.0)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

    let phi = 2.0 * PI * r2;
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let x = sin_theta * cos_phi;
    let y = sin_theta * sin_phi;
    let z = cos_theta;

    let tbn = create_orthonormal_basis(normal);
    tbn.mul_vec3a(Vec3A::new(x, y, z))
}

fn gtr1_importance_sample(normal: Vec3A, gloss: f32) -> Vec3A {
    let r1: f32 = rand::random::<f32>();
    let r2 = rand::random::<f32>();

    let alpha = lerp(0.1, 0.001, gloss);
    let alpha2 = alpha * alpha;

    let cos_theta_sq = (1.0 - alpha2.powf(1.0 - r1)) / (1.0 - alpha2);
    let cos_theta = cos_theta_sq.sqrt();
    let sin_theta = (1.0 - cos_theta_sq).max(0.0).sqrt();

    let phi = 2.0 * PI * r2;
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let x = sin_theta * cos_phi;
    let y = sin_theta * sin_phi;
    let z = cos_theta;

    let tbn = create_orthonormal_basis(normal);
    tbn.mul_vec3a(Vec3A::new(x, y, z))
}

fn ggx_pdf_specular(n_dot_h: f32, v_dot_h: f32, roughness: f32) -> f32 {
    distribution_term_specular(n_dot_h, roughness) * n_dot_h / (4.0 * v_dot_h)
}

fn ggx_pdf_clearcoat(n_dot_h: f32, v_dot_h: f32, gloss: f32) -> f32 {
    distribution_term_clearcoat(n_dot_h, gloss) * n_dot_h / (4.0 * v_dot_h)
}
