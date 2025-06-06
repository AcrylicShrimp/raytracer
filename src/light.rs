use crate::hit::HitRecord;
use glam::Vec3A;

pub trait Light {
    fn sample(&self, hit: &HitRecord) -> Vec3A;
}
