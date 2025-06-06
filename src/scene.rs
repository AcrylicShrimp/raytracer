use crate::{hit::HitRecord, light::Light, object::Object, ray::Ray};
use std::f32;

pub struct Scene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn Light>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            lights: vec![],
        }
    }

    pub fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }

    pub fn lights(&self) -> &[Box<dyn Light>] {
        &self.lights
    }

    pub fn add_object(&mut self, object: impl Object + 'static) {
        self.objects.push(Box::new(object));
    }

    pub fn add_light(&mut self, light: impl Light + 'static) {
        self.lights.push(Box::new(light));
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        let mut closest_t = t_max;

        // Check each object for intersection
        for object in &self.objects {
            if !object.bounding_box().is_intersecting(ray) {
                continue;
            }

            if let Some(hit_record) = object.intersect(ray, t_min, closest_t) {
                closest_t = hit_record.t;
                closest_hit = Some(hit_record);
            }
        }

        closest_hit
    }
}
