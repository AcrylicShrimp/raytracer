use crate::{aabb::Aabb, hit::HitRecord, ray::Ray};

pub trait Object {
    fn bounding_box(&self) -> Aabb;
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
