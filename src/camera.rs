use crate::{ray::Ray, scene::Scene};
use glam::Vec3A;
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
        let fov_rad = self.fov.to_radians();
        let f = 1.0 / (fov_rad / 2.0).tan();

        let pixel_x = (pixel_x * 2.0 - 1.0) * aspect_ratio * f;
        let pixel_y = (pixel_y * 2.0 - 1.0) * f;

        let right = self.direction.cross(self.up).normalize();
        let direction = self.direction - self.up * pixel_y + right * pixel_x;

        Ray::new(self.position, direction)
    }

    pub fn render(
        &self,
        scene: &Scene,
        screen_width: u32,
        screen_height: u32,
        ambient_light: Vec3A,
        exposure: f32,
        gamma: f32,
    ) -> Vec<u8> {
        let aspect_ratio = screen_width as f32 / screen_height as f32;
        let mut hdr_buffer = vec![Vec3A::ZERO; (screen_width * screen_height) as usize];

        hdr_buffer
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index % screen_width as usize;
                let y = index / screen_width as usize;

                const SAMPLES: u32 = 128;
                let mut sdr_sum = Vec3A::ZERO;

                for _ in 0..SAMPLES {
                    let pixel_x = (x as f32 + rand::random::<f32>()) / screen_width as f32;
                    let pixel_y = (y as f32 + rand::random::<f32>()) / screen_height as f32;
                    let ray = self.cast_ray(aspect_ratio, pixel_x, pixel_y);
                    let energy = trace_ray(&ray, scene, 32, ambient_light);
                    sdr_sum += map_hdr_to_sdr(energy, exposure, gamma);
                }

                let color = sdr_sum / SAMPLES as f32;
                *pixel = color;
            });

        let mut frame_buffer = vec![0u8; (screen_width * screen_height * 4) as usize];

        frame_buffer
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(index, pixel)| {
                let color = hdr_buffer[index];
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

fn trace_ray(ray: &Ray, scene: &Scene, depth: u32, ambient_light: Vec3A) -> Vec3A {
    if depth == 0 {
        return Vec3A::ZERO;
    }

    let hit = scene.hit(ray, 1e-3, f32::INFINITY);
    let hit = match hit {
        Some(hit) if hit.front_face => hit,
        _ => {
            return Vec3A::ZERO;
        }
    };
    let albedo = hit.material.albedo;
    let metallic = hit.material.metallic;
    let roughness = hit.material.roughness;

    let mut diffuse = albedo * ambient_light;

    for light in scene.lights() {
        let lit = light.sample(hit.point);
        let shadow_ray_origin = hit.point + hit.normal * 1e-3;
        let shadow_ray = Ray::new(shadow_ray_origin, -lit.direction);
        let is_obstacle_exist = scene.hit(&shadow_ray, 1e-3, lit.distance).is_some();

        if !is_obstacle_exist {
            let diffuse_strength = hit.normal.dot(-lit.direction).max(0f32);
            diffuse += albedo * lit.contribution * diffuse_strength;
        }
    }

    let mut reflection = Vec3A::ZERO;

    if hit.material.is_reflective {
        let reflection_dir = ray.direction.reflect(hit.normal);
        let reflection_ray = Ray::new(hit.point + reflection_dir * 1e-4, reflection_dir);
        reflection = trace_ray(&reflection_ray, scene, depth - 1, ambient_light);
    }

    (diffuse * roughness).lerp(reflection * albedo, metallic)
}

fn map_hdr_to_sdr(color: Vec3A, exposure: f32, gamma: f32) -> Vec3A {
    let color = color * exposure;
    let color = color / (color + 1f32);
    color.powf(1f32 / gamma)
}
