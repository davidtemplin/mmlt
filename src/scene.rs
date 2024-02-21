use std::fs::File;
use std::io;

use serde::{Deserialize, Serialize};

use crate::image::ImageConfig;
use crate::light::LightConfig;
use crate::object::ObjectConfig;
use crate::{
    camera::{Camera, CameraConfig},
    intersection::Intersection,
    light::Light,
    object::Object,
    ray::Ray,
    sampler::Sampler,
};

pub struct Scene {
    pub camera: Box<dyn Camera>,
    pub lights: Vec<Box<dyn Light>>,
    pub objects: Vec<Box<dyn Object>>,
    pub x_resolution: usize,
    pub y_resolution: usize,
}

impl SceneConfig {
    pub fn configure(self: SceneConfig) -> Scene {
        let camera = Box::new(self.camera.configure());
        let lights = self.lights.iter().map(|c| c.configure()).collect();
        let objects = self.objects.iter().map(|c| c.configure()).collect();
        Scene {
            camera,
            lights,
            objects,
            x_resolution: self.image.width,
            y_resolution: self.image.height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SceneConfig {
    image: ImageConfig,
    camera: CameraConfig,
    lights: Vec<LightConfig>,
    objects: Vec<ObjectConfig>,
}

impl Scene {
    pub fn load(path: String) -> Result<Scene, String> {
        let file = File::open(path).map_err(|e: io::Error| e.to_string())?;
        let config: SceneConfig =
            serde_yaml::from_reader(file).map_err(|e: serde_yaml::Error| e.to_string())?;
        println!("{}", format!("{config:#?}"));
        let scene = config.configure();
        Ok(scene)
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
