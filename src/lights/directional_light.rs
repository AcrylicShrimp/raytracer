use crate::{hit::HitRecord, light::Light};
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub color: Vec3A,
    pub intensity: f32,
    pub direction: Vec3A,
}

impl Light for DirectionalLight {
    fn sample(&self, hit: &HitRecord) -> Vec3A {
        let light_direction = self.direction.normalize();
        let light_intensity = self.intensity;
        let light_color = self.color;

        let dot_product = (-light_direction).dot(hit.normal).max(0f32);
        let intensity = light_intensity * dot_product;

        light_color * intensity
    }
}
