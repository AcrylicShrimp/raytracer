use crate::object::Object;
use glam::Vec3A;

#[derive(Clone)]
pub struct HitRecord<'a> {
    pub point: Vec3A,     // The intersection point
    pub normal: Vec3A,    // The surface normal at the intersection point
    pub t: f32,           // The ray parameter (distance)
    pub front_face: bool, // Whether the ray hit the front face
    pub object: &'a dyn Object,
    pub object_index: usize,
}

impl<'a> HitRecord<'a> {
    pub fn new(
        point: Vec3A,
        outward_normal: Vec3A,
        t: f32,
        ray_direction: Vec3A,
        object: &'a dyn Object,
        object_index: usize,
    ) -> Self {
        // Determine if the ray is hitting from outside or inside the object
        let front_face = ray_direction.dot(outward_normal) < 0.0;

        // Ensure the normal always points against the ray
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            point,
            normal,
            t,
            front_face,
            object,
            object_index,
        }
    }
}
