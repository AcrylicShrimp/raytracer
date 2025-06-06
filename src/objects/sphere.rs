use crate::{aabb::Aabb, hit::HitRecord, material::Material, object::Object, ray::Ray};
use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material: Material,
}

impl Object for Sphere {
    fn bounding_box(&self) -> Aabb {
        Aabb {
            min: self.center - Vec3A::splat(self.radius),
            max: self.center + Vec3A::splat(self.radius),
        }
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        // Optimized quadratic formula calculation
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        // Find the nearest root that lies in the acceptable range
        let sqrt_discriminant = discriminant.sqrt();

        // Try the smaller t value first
        let mut t = (-half_b - sqrt_discriminant) / a;

        // Check if t is within valid range
        if t < t_min || t > t_max {
            // Try the larger t value
            t = (-half_b + sqrt_discriminant) / a;
            if t < t_min || t > t_max {
                // Both t values are outside valid range
                return None;
            }
        }

        let point = ray.origin + ray.direction * t;
        let outward_normal = (point - self.center).normalize();

        // Create hit record with the proper normal orientation
        let hit_record = HitRecord::new(
            point,
            outward_normal,
            t,
            ray.direction,
            self.material.clone(),
        );

        Some(hit_record)
    }
}
