use crate::hit::LitRecord;
use glam::Vec3A;

pub trait Light: Send + Sync {
    fn sample(&self, position: Vec3A) -> LitRecord;
}
