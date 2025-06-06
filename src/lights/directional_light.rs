use crate::{hit::LitRecord, light::Light};
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub color: Vec3A,
    pub intensity: f32,
    pub direction: Vec3A,
}

impl Light for DirectionalLight {
    fn sample(&self, _position: Vec3A) -> LitRecord {
        LitRecord {
            contribution: self.color * self.intensity,
            direction: self.direction.normalize(),
            distance: f32::INFINITY,
        }
    }
}
