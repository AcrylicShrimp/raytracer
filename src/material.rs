use glam::Vec3A;

#[derive(Debug, Clone)]
pub struct Material {
    pub is_reflective: bool,
    pub albedo: Vec3A,
    pub subsurface: f32,
    pub metallic: f32,
    pub specular: f32,
    pub specular_tint: Vec3A,
    pub roughness: f32,
    pub anisotropic: f32,
    pub sheen: f32,
    pub sheen_tint: Vec3A,
    pub clearcoat: f32,
    pub clearcoat_gloss: f32,
}
