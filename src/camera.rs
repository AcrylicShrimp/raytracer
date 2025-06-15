use crate::{
    brdf::{Brdf, BrdfEval},
    hit::HitRecord,
    ray::Ray,
    scene::Scene,
};
use glam::Vec3A;
use rand::seq::IndexedRandom;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3A,
    pub direction: Vec3A,
    pub up: Vec3A,
    pub fov: f32,
}

impl Camera {
    pub fn look_at(position: Vec3A, target: Vec3A, up: Vec3A, fov: f32) -> Self {
        let direction = (target - position).normalize();
        let right = direction.cross(up).normalize();
        let up = right.cross(direction).normalize();

        Self {
            position,
            direction,
            up,
            fov,
        }
    }

    pub fn cast_ray(&self, aspect_ratio: f32, pixel_x: f32, pixel_y: f32) -> Ray {
        let ndc_x = pixel_x * 2.0 - 1.0;
        let ndc_y = 1.0 - pixel_y * 2.0;

        let tan_fov_half = (self.fov.to_radians() / 2.0).tan();
        let plane_x = ndc_x * aspect_ratio * tan_fov_half;
        let plane_y = ndc_y * tan_fov_half;

        let right = self.direction.cross(self.up).normalize();
        let up = right.cross(self.direction).normalize();

        let direction = (self.direction + right * plane_x + up * plane_y).normalize();

        Ray::new(self.position, direction)
    }

    pub fn render(&self, scene: &Scene, brdf: &impl Brdf, options: &RenderOptions) -> Vec<u8> {
        let screen_width = options.screen_width;
        let screen_height = options.screen_height;
        let aspect_ratio = screen_width as f32 / screen_height as f32;
        let sample_per_pixel = options.sample_per_pixel;
        let max_ray_bounces = options.max_ray_bounces;
        let exposure = options.exposure;
        let gamma = options.gamma;

        let mut buffer = vec![Vec3A::ZERO; (screen_width * screen_height) as usize];

        buffer
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index % screen_width as usize;
                let y = index / screen_width as usize;

                let mut color = Vec3A::ZERO;

                for _ in 0..sample_per_pixel {
                    let pixel_x = (x as f32 + rand::random::<f32>()) / screen_width as f32;
                    let pixel_y = (y as f32 + rand::random::<f32>()) / screen_height as f32;
                    let ray = self.cast_ray(aspect_ratio, pixel_x, pixel_y);
                    let energy = trace_ray(&ray, scene, brdf, max_ray_bounces, None);
                    color += energy / sample_per_pixel as f32;
                }

                *pixel = map_hdr_to_sdr(color, exposure, gamma);
            });

        let mut frame_buffer = vec![0u8; (screen_width * screen_height * 4) as usize];

        frame_buffer
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(index, pixel)| {
                let color = buffer[index];
                let color = (color * 255f32)
                    .clamp(Vec3A::ZERO, Vec3A::splat(255f32))
                    .round();
                pixel[0] = color.x as u8;
                pixel[1] = color.y as u8;
                pixel[2] = color.z as u8;
                pixel[3] = 255;
            });

        frame_buffer
    }
}

fn map_hdr_to_sdr(color: Vec3A, exposure: f32, gamma: f32) -> Vec3A {
    let color = color * exposure;
    let color = color / (color + 1f32);
    color.powf(1f32 / gamma)
}

pub struct RenderOptions {
    pub screen_width: u32,
    pub screen_height: u32,
    pub sample_per_pixel: u32,
    pub max_ray_bounces: u32,
    pub exposure: f32,
    pub gamma: f32,
}

/// Solves the rendering equation for a given ray, using the BRDF:
///
/// `L_o = L_e + f_r * L_i * (N dot L) / pdf`
///
/// where:
/// - `L_o` is the outgoing radiance
/// - `L_e` is the emitted radiance
/// - `f_r` is the BRDF
/// - `L_i` is the incoming radiance
/// - `N` is the surface normal
/// - `L` is the light direction
/// - `pdf` is the probability density function of the BRDF
///
/// The function returns the outgoing radiance `L_o`.
///
/// The function is recursive, and terminates when the depth limit is reached.
///
/// Note that the BRDF is responsible for computing `attenuation`, which represents:
///
/// `attenuation = f_r * cos_theta / pdf`
fn trace_ray<'a>(
    ray: &Ray,
    scene: &'a Scene,
    brdf: &impl Brdf,
    depth: u32,
    hit: Option<HitRecord<'a>>,
) -> Vec3A {
    if depth == 0 {
        return Vec3A::ZERO;
    }

    let hit = hit.or_else(|| scene.hit(ray, 1e-3, f32::INFINITY));
    let hit = match hit {
        Some(hit) if hit.front_face => hit,
        _ => {
            return Vec3A::ZERO;
        }
    };
    let is_delta_surface = brdf.is_delta_surface(hit.object.material());

    if hit.object.material().is_emissive {
        return hit.object.material().emission;
    }

    let direct_term = if is_delta_surface {
        Vec3A::ZERO
    } else {
        compute_nee_contribution(&hit, scene, brdf, -ray.direction)
    };

    let brdf_sample = brdf.sample(-ray.direction, hit.normal, hit.object.material());

    if brdf_sample.attenuation.length_squared() < 1e-5 || brdf_sample.pdf < 1e-5 {
        // indirect term is too small; ignore it
        return direct_term;
    }

    let next_ray = Ray::new(hit.point + hit.normal * 1e-3, brdf_sample.direction);
    let next_hit = scene.hit(&next_ray, 1e-3, f32::INFINITY);
    let indirect_term = match next_hit {
        Some(next_hit) if next_hit.object.material().is_emissive && is_delta_surface => {
            next_hit.object.material().emission * brdf_sample.attenuation
        }
        Some(next_hit) if next_hit.object.material().is_emissive && !is_delta_surface => {
            let pdf_brdf = brdf_sample.pdf;
            let r_squared = (next_hit.point - hit.point).length_squared();
            let cos_theta_l = next_hit.normal.dot(-next_ray.direction).max(0.0);

            if cos_theta_l < 1e-5 {
                Vec3A::ZERO
            } else {
                let light_area = next_hit.object.area();
                let n_light = scene.light_count() as f32;
                let pdf_light = (r_squared / (cos_theta_l * light_area)) / n_light;

                let mis_weight_brdf = pdf_brdf / (pdf_light + pdf_brdf);
                next_hit.object.material().emission * brdf_sample.attenuation * mis_weight_brdf
            }
        }
        Some(next_hit) => {
            trace_ray(&next_ray, scene, brdf, depth - 1, Some(next_hit)) * brdf_sample.attenuation
        }
        None => Vec3A::ZERO,
    };

    // L_o * attenuation
    direct_term + indirect_term
}

fn compute_nee_contribution(
    hit: &HitRecord,
    scene: &Scene,
    brdf: &impl Brdf,
    view: Vec3A,
) -> Vec3A {
    let total_light_objects: Vec<_> = scene
        .objects()
        .iter()
        .filter(|object| object.material().is_emissive)
        .collect();
    let chosen_light_object = total_light_objects.choose(&mut rand::rng());
    let light_object = match chosen_light_object {
        Some(light_object) => light_object.as_ref(),
        None => {
            // no light objects; ignore it
            return Vec3A::ZERO;
        }
    };

    let n_light = scene.light_count();
    let n_light_inv = (n_light as f32).recip();

    let area = light_object.area();
    let area_inv = area.recip();

    if area < 1e-5 {
        // area is too small; ignore it
        return Vec3A::ZERO;
    }

    let light_point = light_object.sample_point();
    let diff = light_point.point - hit.point;

    if diff.length_squared() < 1e-3 {
        // light is too close; ignore it, treating the light as if it is behind the surface
        return Vec3A::ZERO;
    }

    let r_squared = diff.length_squared();
    let r = diff.length();
    let light_direction = diff / r;

    let cos_theta = hit.normal.dot(light_direction).max(0.0);
    let cos_theta_l = light_point.normal.dot(-light_direction).max(0.0);

    if cos_theta_l < 1e-5 {
        // light is not visible; ignore it
        return Vec3A::ZERO;
    }

    let shadow_ray = Ray::new(hit.point + hit.normal * 1e-3, light_direction);
    let is_visible = scene.hit(&shadow_ray, 1e-3, r - 1e-3).is_none();

    if !is_visible {
        // light is not visible; ignore it
        return Vec3A::ZERO;
    }

    let BrdfEval { f_r, pdf: pdf_brdf } =
        brdf.eval(view, hit.normal, light_direction, hit.object.material());
    let pdf_light = r_squared / cos_theta_l * area_inv * n_light_inv;

    if pdf_brdf < 1e-5 && pdf_light < 1e-5 {
        // pdf is too small; ignore it
        return Vec3A::ZERO;
    }

    let mis_weight = pdf_light / (pdf_brdf + pdf_light);
    let geometry_term = cos_theta * cos_theta_l / r_squared;
    let contribution = light_object.material().emission * f_r * geometry_term;
    let pdf_area = area_inv * n_light_inv;

    (contribution / pdf_area) * mis_weight
}
