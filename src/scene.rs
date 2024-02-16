use crate::{
    camera::Camera, intersection::Intersection, light::Light, object::Object, ray::Ray,
    sampler::Sampler,
};

pub struct Scene {
    pub camera: Box<dyn Camera>,
    pub lights: Vec<Box<dyn Light>>,
    pub objects: Vec<Box<dyn Object>>,
}

impl Scene {
    pub fn new() -> Scene {
        todo!()
    }

    pub fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let mut result: Option<Intersection> = None;

        if let Some(candidate) = self.camera.intersect(ray) {
            if let Some(ref best) = result {
                if candidate.distance() < best.distance() {
                    result = Some(candidate);
                }
            } else {
                result = Some(candidate);
            }
        }

        for light in &self.lights {
            if let Some(candidate) = light.intersect(ray) {
                if let Some(ref best) = result {
                    if candidate.distance() < best.distance() {
                        result = Some(candidate);
                    }
                } else {
                    result = Some(candidate);
                }
            }
        }

        for object in &self.objects {
            if let Some(candidate) = object.intersect(ray) {
                if let Some(ref best) = result {
                    if candidate.distance() < best.distance() {
                        result = Some(candidate);
                    }
                } else {
                    result = Some(candidate);
                }
            }
        }

        result
    }

    pub fn sample_light(&self, sampler: &mut impl Sampler) -> &(dyn Light) {
        let start = 0.0;
        let end = self.lights.len() as f64;
        let r = sampler.sample(start..end);
        let i = r.floor() as usize;
        self.lights[i].as_ref()
    }
}
