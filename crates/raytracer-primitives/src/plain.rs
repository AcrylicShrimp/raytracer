use glam::{Mat3A, Vec2, Vec3A};
use raytracer_core::{
    aabb::Aabb,
    hit_record::HitRecord,
    material::Material,
    object::{Object, PointOnObject},
    ray::Ray,
};

#[derive(Debug, Clone)]
pub struct Plain {
    pub center: Vec3A,
    pub normal: Vec3A,
    pub size: Vec2,
    pub material: Material,
}

impl Plain {
    pub fn rotation(&self) -> Mat3A {
        let new_z = self.normal;
        let temp_up = if new_z.x.abs() > 0.9 || new_z.z.abs() > 0.9 {
            Vec3A::Y
        } else {
            Vec3A::X
        };

        let new_x = temp_up.cross(new_z).normalize();
        let new_y = new_z.cross(new_x).normalize();

        Mat3A::from_cols(new_x, new_y, new_z)
    }
}

impl Object for Plain {
    fn material(&self) -> &Material {
        &self.material
    }

    fn area(&self) -> f32 {
        self.size.x * self.size.y
    }

    fn sample_point(&self) -> PointOnObject {
        let u = rand::random::<f32>();
        let v = rand::random::<f32>();

        let local_point = Vec3A::new((u - 0.5) * self.size.x, (v - 0.5) * self.size.y, 0.0);
        let world_point = self.center + self.rotation().mul_vec3a(local_point);

        PointOnObject {
            point: world_point,
            normal: self.normal,
        }
    }

    fn bounding_box(&self) -> Aabb {
        let half_size = Vec3A::new(self.size.x * 0.5, self.size.y * 0.5, 1e-3);

        let rot_mat = self.rotation();
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
        let denominator = self.normal.dot(ray.direction);

        if denominator.abs() < 1e-5 {
            return None;
        }

        let t = (self.center - ray.origin).dot(self.normal) / denominator;

        if t < t_min || t > t_max {
            return None;
        }

        let hit_point = ray.origin + ray.direction * t;
        let vector_from_center = hit_point - self.center;

        let rotation = self.rotation();
        let local_x_axis = rotation.x_axis;
        let local_y_axis = rotation.y_axis;

        let local_x = vector_from_center.dot(local_x_axis);
        let local_y = vector_from_center.dot(local_y_axis);

        let half_size = self.size * 0.5;
        if local_x.abs() > half_size.x || local_y.abs() > half_size.y {
            return None;
        }

        Some(HitRecord::new(
            hit_point,
            self.normal,
            t,
            ray.direction,
            self,
            object_index,
        ))
    }
}
