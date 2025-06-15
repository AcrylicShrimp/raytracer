use crate::{
    brdf::{Brdf, BrdfEval, BrdfSample, create_orthonormal_basis, lerp, random_cosine_direction},
    material::Material,
};
use glam::Vec3A;
use std::f32::consts::{FRAC_1_PI, PI};

pub struct Disney;

impl Brdf for Disney {
    fn is_delta_surface(&self, material: &Material) -> bool {
        material.roughness < 1e-3
    }

    fn eval(&self, view: Vec3A, normal: Vec3A, light: Vec3A, material: &Material) -> BrdfEval {
        if normal.dot(light) <= 0.0 {
            return BrdfEval::ZERO;
        }

        let half = (view + light).normalize();
        let n_dot_h = normal.dot(half);
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
        let specular_term =
            specular_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.roughness, f0);
        let clearcoat_term =
            clearcoat_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.clearcoat_gloss);

        let f_r = (1.0 - material.metallic) * diffuse_term + specular_term + clearcoat_term;

        let clearcoat_prob = material.clearcoat / (1.0 + material.clearcoat);
        let base_prob = 1.0 - clearcoat_prob;

        let pdf_clearcoat = ggx_pdf_clearcoat(n_dot_h, view.dot(half), material.clearcoat_gloss);
        let weighted_pdf_clearcoat = pdf_clearcoat * clearcoat_prob;

        let pdf_specular = ggx_pdf_specular(n_dot_h, view.dot(half), material.roughness);
        let pdf_diffuse = n_dot_l.max(0.0) * FRAC_1_PI;

        let weighted_pdf_base = if material.metallic < 1.0 {
            let f = fresnel_term(l_dot_h, Vec3A::splat(material.specular * 0.08));
            let specular_prob = f.max_element();

            let pdf_dielectric_specular = pdf_specular * specular_prob;
            let pdf_dielectric_diffuse = pdf_diffuse * (1.0 - specular_prob);

            (1.0 - material.metallic) * (pdf_dielectric_specular + pdf_dielectric_diffuse)
        } else {
            pdf_specular * material.metallic
        };

        let pdf = weighted_pdf_clearcoat + weighted_pdf_base * base_prob;

        BrdfEval { f_r, pdf }
    }

    fn sample(&self, view: Vec3A, normal: Vec3A, material: &Material) -> BrdfSample {
        let clearcoat_prob = material.clearcoat / (1.0 + material.clearcoat);

        if rand::random::<f32>() < clearcoat_prob {
            let half = gtr1_importance_sample(normal, material.clearcoat_gloss);
            let light = (-view).reflect(half);

            let l_dot_h = light.dot(half);
            let n_dot_h = normal.dot(half);
            let n_dot_l = normal.dot(light);
            let n_dot_v = normal.dot(view);
            let v_dot_h = view.dot(half);

            let pdf =
                ggx_pdf_clearcoat(n_dot_h, v_dot_h, material.clearcoat_gloss) * clearcoat_prob;

            if pdf < 1e-5 {
                return BrdfSample::ZERO;
            }

            if n_dot_l <= 0.0 {
                return BrdfSample::ZERO;
            }

            let attenuation =
                clearcoat_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.clearcoat_gloss)
                    * n_dot_l
                    / pdf;

            return BrdfSample {
                direction: light,
                attenuation,
                pdf,
            };
        }

        if material.roughness < 1e-3 {
            // perfect reflection
            let direction = (-view).reflect(normal);

            let dielectric_f0 = Vec3A::splat(material.specular * 0.08);
            let metallic_f0 = material.albedo;
            let f0 = dielectric_f0.lerp(metallic_f0, material.metallic);

            let pdf = 1.0 - clearcoat_prob;

            if pdf < 1e-5 {
                return BrdfSample::ZERO;
            }

            let attenuation = fresnel_term(direction.dot(normal).max(0.0), f0) / pdf;

            return BrdfSample {
                direction,
                attenuation,
                pdf,
            };
        }

        if rand::random::<f32>() < material.metallic {
            let half = gtr2_importance_sample(normal, material.roughness);
            let light = (-view).reflect(half);

            let n_dot_h = normal.dot(half);
            let n_dot_v = normal.dot(view);
            let n_dot_l = normal.dot(light);
            let v_dot_h = view.dot(half);
            let l_dot_h = light.dot(half);

            let pdf = ggx_pdf_specular(n_dot_h, v_dot_h, material.roughness)
                * (1.0 - clearcoat_prob)
                * material.metallic;

            if pdf < 1e-5 {
                return BrdfSample::ZERO;
            }

            if n_dot_l <= 0.0 {
                return BrdfSample::ZERO;
            }

            let dielectric_f0 = Vec3A::splat(material.specular * 0.08);
            let metallic_f0 = material.albedo;
            let f0 = dielectric_f0.lerp(metallic_f0, material.metallic);
            let attenuation =
                specular_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.roughness, f0) * n_dot_l
                    / pdf;

            return BrdfSample {
                direction: light,
                attenuation,
                pdf,
            };
        }

        let half = gtr2_importance_sample(normal, material.roughness);
        let light = (-view).reflect(half);

        let l_dot_h = light.dot(half);

        let f0 = Vec3A::splat(material.specular * 0.08);
        let f = fresnel_term(l_dot_h, f0);

        let specular_prob = f.max_element();

        if rand::random::<f32>() < specular_prob {
            let n_dot_h = normal.dot(half);
            let n_dot_l = normal.dot(light);
            let n_dot_v = normal.dot(view);
            let v_dot_h = view.dot(half);

            let pdf = ggx_pdf_specular(n_dot_h, v_dot_h, material.roughness)
                * (1.0 - clearcoat_prob)
                * (1.0 - material.metallic)
                * specular_prob;

            if pdf < 1e-5 {
                return BrdfSample::ZERO;
            }

            if n_dot_l <= 0.0 {
                return BrdfSample::ZERO;
            }

            let attenuation =
                specular_term(n_dot_h, n_dot_v, n_dot_l, l_dot_h, material.roughness, f0) * n_dot_l
                    / pdf;

            BrdfSample {
                direction: light,
                attenuation,
                pdf,
            }
        } else {
            let light = random_cosine_direction(normal);
            let half = (view + light).normalize();

            let n_dot_v = normal.dot(view);
            let n_dot_l = normal.dot(light);
            let l_dot_h = light.dot(half);

            let pdf = normal.dot(light).max(0.0)
                * FRAC_1_PI
                * (1.0 - clearcoat_prob)
                * (1.0 - material.metallic)
                * (1.0 - specular_prob);

            if pdf < 1e-5 {
                return BrdfSample::ZERO;
            }

            if n_dot_l <= 0.0 {
                return BrdfSample::ZERO;
            }

            let attenuation = diffuse_term(
                n_dot_v,
                n_dot_l,
                l_dot_h,
                material.roughness,
                material.albedo,
            ) * n_dot_l
                / pdf;

            BrdfSample {
                direction: light,
                attenuation,
                pdf,
            }
        }
    }
}

fn distribution_term_specular(n_dot_h: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let denom_core = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let denom = std::f32::consts::PI * denom_core * denom_core;

    alpha2 / denom.max(1e-3)
}

fn distribution_term_clearcoat(n_dot_h: f32, gloss: f32) -> f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    let alpha = lerp(0.1, 0.001, gloss);
    let alpha2 = alpha * alpha;

    let denom_core = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let c = if (alpha2 - 1.0).abs() < 1e-3 {
        1.0 / std::f32::consts::PI
    } else {
        (alpha2 - 1.0) / (std::f32::consts::PI * alpha2.ln())
    };

    c / denom_core.max(1e-3)
}

fn fresnel_term(l_dot_h: f32, f0: Vec3A) -> Vec3A {
    f0 + (Vec3A::ONE - f0) * (1.0 - l_dot_h).powf(5.0)
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

fn specular_term(
    n_dot_h: f32,
    n_dot_v: f32,
    n_dot_l: f32,
    l_dot_h: f32,
    roughness: f32,
    f0: Vec3A,
) -> Vec3A {
    distribution_term_specular(n_dot_h, roughness)
        * fresnel_term(l_dot_h, f0)
        * geometry_term(n_dot_v, n_dot_l, roughness)
        / (4.0 * n_dot_v * n_dot_l)
}

fn clearcoat_term(n_dot_h: f32, n_dot_v: f32, n_dot_l: f32, l_dot_h: f32, gloss: f32) -> Vec3A {
    distribution_term_clearcoat(n_dot_h, gloss)
        * fresnel_term(l_dot_h, Vec3A::splat(0.04))
        * geometry_term(n_dot_v, n_dot_l, 0.25)
        / (4.0 * n_dot_v * n_dot_l)
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

fn ggx_pdf_specular(n_dot_h: f32, v_dot_h: f32, roughness: f32) -> f32 {
    distribution_term_specular(n_dot_h, roughness) * n_dot_h / (4.0 * v_dot_h)
}

fn ggx_pdf_clearcoat(n_dot_h: f32, v_dot_h: f32, gloss: f32) -> f32 {
    distribution_term_clearcoat(n_dot_h, gloss) * n_dot_h / (4.0 * v_dot_h)
}
