use crate::{
    aabb::Aabb,
    hit::HitRecord,
    material::Material,
    object::{Object, PointOnObject},
    ray::Ray,
};
use glam::{Mat3A, Quat, Vec3A};

#[derive(Debug, Clone)]
pub struct Box {
    pub center: Vec3A,
    pub size: Vec3A,
    pub rotation: Quat,
    pub material: Material,
}

impl Object for Box {
    fn material(&self) -> &Material {
        &self.material
    }

    fn area(&self) -> f32 {
        self.size.x * self.size.y * 2.0
            + self.size.x * self.size.z * 2.0
            + self.size.y * self.size.z * 2.0
    }

    fn sample_point(&self) -> PointOnObject {
        let area = self.area();

        if area < 1e-5 {
            return PointOnObject {
                point: self.center,
                normal: Vec3A::Y,
            };
        }

        let size_x = self.size.y * self.size.z * 2.0;
        let size_y = self.size.x * self.size.z * 2.0;
        let size_z = self.size.x * self.size.y * 2.0;

        let area_inv = area.recip();
        let p_x_plus = size_x * area_inv * 0.5;
        let p_x_minus = size_x * area_inv * 0.5;
        let p_y_plus = size_y * area_inv * 0.5;
        let p_y_minus = size_y * area_inv * 0.5;
        let p_z_plus = size_z * area_inv * 0.5;

        let cdf_x_minus = p_x_plus + p_x_minus;
        let cdf_y_plus = cdf_x_minus + p_y_plus;
        let cdf_y_minus = cdf_y_plus + p_y_minus;
        let cdf_z_plus = cdf_y_minus + p_z_plus;

        let u = rand::random::<f32>();
        let v = rand::random::<f32>();
        let dice = rand::random::<f32>();

        let (local_point, local_normal) = if dice < p_x_plus {
            let y = (u * 2.0 - 1.0) * self.size.y * 0.5;
            let z = (v * 2.0 - 1.0) * self.size.z * 0.5;
            (Vec3A::new(self.size.x * 0.5, y, z), Vec3A::X)
        } else if dice < cdf_x_minus {
            let y = (u * 2.0 - 1.0) * self.size.y * 0.5;
            let z = (v * 2.0 - 1.0) * self.size.z * 0.5;
            (Vec3A::new(-self.size.x * 0.5, y, z), Vec3A::NEG_X)
        } else if dice < cdf_y_plus {
            let x = (u * 2.0 - 1.0) * self.size.x * 0.5;
            let z = (v * 2.0 - 1.0) * self.size.z * 0.5;
            (Vec3A::new(x, self.size.y * 0.5, z), Vec3A::Y)
        } else if dice < cdf_y_minus {
            let x = (u * 2.0 - 1.0) * self.size.x * 0.5;
            let z = (v * 2.0 - 1.0) * self.size.z * 0.5;
            (Vec3A::new(x, -self.size.y * 0.5, z), Vec3A::NEG_Y)
        } else if dice < cdf_z_plus {
            let x = (u * 2.0 - 1.0) * self.size.x * 0.5;
            let y = (v * 2.0 - 1.0) * self.size.y * 0.5;
            (Vec3A::new(x, y, self.size.z * 0.5), Vec3A::Z)
        } else {
            let x = (u * 2.0 - 1.0) * self.size.x * 0.5;
            let y = (v * 2.0 - 1.0) * self.size.y * 0.5;
            (Vec3A::new(x, y, -self.size.z * 0.5), Vec3A::NEG_Z)
        };

        PointOnObject {
            point: self.rotation.mul_vec3a(local_point) + self.center,
            normal: self.rotation.mul_vec3a(local_normal),
        }
    }

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

    fn intersect(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        object_index: usize,
    ) -> Option<HitRecord> {
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
            self,
            object_index,
        ))
    }
}
