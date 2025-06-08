use crate::{aabb::Aabb, hit::HitRecord, material::Material, object::Object, ray::Ray};
use glam::{Mat3A, Quat, Vec3A};

#[derive(Debug, Clone)]
pub struct Box {
    pub center: Vec3A,
    pub size: Vec3A,
    pub rotation: Quat,
    pub material: Material,
}

impl Object for Box {
    fn bounding_box(&self) -> Aabb {
        let half_size = self.size * 0.5;
        let rot_mat = Mat3A::from_quat(self.rotation);
        let abs_rot_mat = Mat3A::from_cols(
            rot_mat.x_axis.abs(),
            rot_mat.y_axis.abs(),
            rot_mat.z_axis.abs(),
        );
        let aabb_half_extent = abs_rot_mat.mul_vec3a(half_size);

        Aabb {
            min: self.center - aabb_half_extent,
            max: self.center + aabb_half_extent,
        }
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let half_size = self.size * 0.5;
        let inv_rotation = self.rotation.inverse();
        let local_ray = Ray {
            origin: inv_rotation.mul_vec3a(ray.origin - self.center),
            direction: inv_rotation.mul_vec3a(ray.direction),
        };

        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;
        let mut hit_normal_local = Vec3A::ZERO;

        for i in 0..3 {
            let inv_d = 1.0 / local_ray.direction[i];
            let mut t1 = (-half_size[i] - local_ray.origin[i]) * inv_d;
            let mut t2 = (half_size[i] - local_ray.origin[i]) * inv_d;

            let mut normal = Vec3A::ZERO;
            normal[i] = -1.0;

            if t2 < t1 {
                std::mem::swap(&mut t1, &mut t2);
                normal = -normal;
            }

            if t_near < t1 {
                t_near = t1;
                hit_normal_local = normal;
            }

            t_far = t_far.min(t2);

            if t_far <= t_near {
                return None;
            }
        }

        let t_hit = if t_min <= t_near && t_near <= t_max {
            t_near
        } else if t_min <= t_far && t_far <= t_max {
            t_far
        } else {
            return None;
        };

        let local_hit_point = local_ray.origin + local_ray.direction * t_hit;
        let world_hit_point = self.rotation.mul_vec3a(local_hit_point) + self.center;
        let world_normal = self.rotation.mul_vec3a(hit_normal_local).normalize();

        Some(HitRecord::new(
            world_hit_point,
            world_normal,
            t_hit,
            ray.direction,
            self.material.clone(),
        ))
    }
}
