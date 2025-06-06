use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Material {
    pub albedo: Vec3A,
    pub metallic: f32,
    pub roughness: f32,
    pub is_reflective: bool,
}
