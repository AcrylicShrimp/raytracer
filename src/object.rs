use crate::{aabb::Aabb, hit::HitRecord, material::Material, ray::Ray};
use glam::Vec3A;

pub trait Object: Send + Sync {
    fn material(&self) -> &Material;
    fn area(&self) -> f32;
    fn sample_point(&self) -> PointOnObject;
    fn bounding_box(&self) -> Aabb;
    fn intersect(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        object_index: usize,
    ) -> Option<HitRecord>;
}

#[derive(Debug, Clone)]
pub struct PointOnObject {
    pub point: Vec3A,
    pub normal: Vec3A,
}
