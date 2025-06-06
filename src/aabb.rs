use crate::ray::Ray;
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Aabb {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl Aabb {
    pub fn is_intersecting(&self, ray: &Ray) -> bool {
        let mut t_min = 0f32;
        let mut t_max = f32::INFINITY;
        let direction_inv = ray.direction.recip();

        for i in 0..3 {
            let t_low = direction_inv[i] * (self.min[i] - ray.origin[i]);
            let t_high = direction_inv[i] * (self.max[i] - ray.origin[i]);

            t_min = t_min.max(t_low.min(t_high));
            t_max = t_max.min(t_low.max(t_high));
        }

        t_min <= t_max
    }
}
