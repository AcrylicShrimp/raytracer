use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}
