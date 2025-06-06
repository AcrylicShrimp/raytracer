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

    pub fn render(&self, scene: &Scene, screen_width: u32, screen_height: u32) -> Vec<u8> {
        let aspect_ratio = screen_width as f32 / screen_height as f32;
        let mut frame_buffer = vec![0; (screen_width * screen_height * 4) as usize];

        for y in 0..screen_height {
            for x in 0..screen_width {
                let pixel_x = (x as f32 + 0.5) / screen_width as f32;
                let pixel_y = (y as f32 + 0.5) / screen_height as f32;
                let ray = self.cast_ray(aspect_ratio, pixel_x, pixel_y);
                let hit = scene.hit(&ray, 0.0, f32::INFINITY);
                let color = match hit {
                    Some(hit) => hit.material.albedo,
                    None => Vec3A::ZERO,
                };

                let pixel_index = ((y * screen_width + x) * 4) as usize;
                frame_buffer[pixel_index] = (color.x * 255.0) as u8;
                frame_buffer[pixel_index + 1] = (color.y * 255.0) as u8;
                frame_buffer[pixel_index + 2] = (color.z * 255.0) as u8;
                frame_buffer[pixel_index + 3] = 255;
            }
        }

        frame_buffer
    }
}
