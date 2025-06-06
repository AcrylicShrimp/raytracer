use crate::hit::HitRecord;
use glam::Vec3A;

pub trait Light: Send + Sync {
    fn sample(&self, hit: &HitRecord) -> Vec3A;
}
