use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Material {
    pub albedo: Vec3A,
    pub roughness: f32,
    pub is_reflective: bool,
}
