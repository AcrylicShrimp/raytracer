use crate::{ray::Ray, scene::Scene};
use glam::Vec3A;

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
        let right = up.cross(direction).normalize();
        let up = direction.cross(right).normalize();

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
        let direction = self.direction + self.up * pixel_y + right * pixel_x;

        Ray {
            origin: self.position,
            direction,
        }
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
        let mut hdr_frame_buffer = vec![Vec3A::ZERO; (screen_width * screen_height) as usize];

        for y in 0..screen_height {
            for x in 0..screen_width {
                const SAMPLES: u32 = 64;
                let mut sum = Vec3A::ZERO;

                for _ in 0..SAMPLES {
                    let pixel_x = (x as f32 + rand::random::<f32>()) / screen_width as f32;
                    let pixel_y = (y as f32 + rand::random::<f32>()) / screen_height as f32;
                    let ray = self.cast_ray(aspect_ratio, pixel_x, pixel_y);
                    let hit = scene.hit(&ray, 1e-3, f32::INFINITY);
                    let hit = match hit {
                        Some(hit) if hit.front_face => hit,
                        _ => {
                            sum += Vec3A::ZERO;
                            continue;
                        }
                    };

                    let mut final_energy = hit.material.albedo * ambient_light;

                    for light in scene.lights() {
                        let contribution = light.sample(&hit);
                        final_energy += hit.material.albedo * contribution;
                    }

                    sum += final_energy;
                }

                let color = sum / SAMPLES as f32;
                hdr_frame_buffer[(y * screen_width + x) as usize] = color;
            }
        }

        let mut sdr_frame_buffer = vec![0u8; (screen_width * screen_height * 4) as usize];

        for index in 0..hdr_frame_buffer.len() {
            let color = hdr_frame_buffer[index];
            let color = color * exposure;
            let color = color / (color + Vec3A::splat(1f32));
            let color = color.powf(1f32 / gamma);
            let color = (color * Vec3A::splat(255f32))
                .clamp(Vec3A::ZERO, Vec3A::splat(255f32))
                .round();
            sdr_frame_buffer[index * 4] = color.x as u8;
            sdr_frame_buffer[index * 4 + 1] = color.y as u8;
            sdr_frame_buffer[index * 4 + 2] = color.z as u8;
            sdr_frame_buffer[index * 4 + 3] = 255;
        }

        sdr_frame_buffer
    }
}
