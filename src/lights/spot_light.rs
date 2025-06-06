use crate::{hit::HitRecord, light::Light};
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct SpotLight {
    pub color: Vec3A,
    pub intensity: f32,
    pub position: Vec3A,
}

impl Light for SpotLight {
    fn sample(&self, hit: &HitRecord) -> Vec3A {
        if self.position.distance_squared(hit.point) <= 1e-3 {
            return self.color * self.intensity;
        }

        let light_direction = (self.position - hit.point).normalize();
        let light_intensity = self.intensity;
        let light_color = self.color;

        let dot_product = (-light_direction).dot(hit.normal).max(0f32);
        let intensity = light_intensity * dot_product;

        light_color * intensity
    }
}
