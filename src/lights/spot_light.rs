use crate::{hit::LitRecord, light::Light};
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct SpotLight {
    pub color: Vec3A,
    pub intensity: f32,
    pub position: Vec3A,
}

impl Light for SpotLight {
    fn sample(&self, position: Vec3A) -> LitRecord {
        let distance = self.position.distance(position);

        if distance <= 1e-3 {
            return LitRecord {
                contribution: self.color * self.intensity,
                direction: Vec3A::ONE,
                distance,
            };
        }

        LitRecord {
            contribution: self.color * self.intensity * (1f32 + distance).powf(2.0).recip(),
            direction: (position - self.position).normalize(),
            distance,
        }
    }
}
