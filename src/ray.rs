use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}

impl Ray {
    pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
}
