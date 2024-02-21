use serde::{Deserialize, Serialize};

use crate::{
    intersection::Intersection,
    ray::Ray,
    sampler::Sampler,
    shape::{Shape, ShapeConfig},
    spectrum::{Spectrum, SpectrumConfig},
    vector::Vector,
};

pub trait Light {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum;
    fn probability(&self, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection<'_>>;
    fn id(&self) -> u64;
}

pub struct DiffuseAreaLight {
    id: u64,
    shape: Box<dyn Shape>,
    radiance: Spectrum,
}

impl Light for DiffuseAreaLight {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum {
        if normal.dot(direction) > 0.0 {
            self.radiance
        } else {
            Spectrum::black()
        }
    }

    fn probability(&self, direction: Vector) -> f64 {
        self.shape.probability(direction)
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        self.shape.sample_intersection(sampler)
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        self.shape.intersect(ray)
    }

    fn id(&self) -> u64 {
        self.id
    }
}

impl DiffuseAreaLight {
    pub fn configure(config: &DiffuseAreaLightConfig) -> DiffuseAreaLight {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum LightConfig {
    DiffuseArea(DiffuseAreaLightConfig),
}

impl LightConfig {
    pub fn configure(&self) -> Box<dyn Light> {
        match self {
            LightConfig::DiffuseArea(config) => Box::new(DiffuseAreaLight::configure(config)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiffuseAreaLightConfig {
    pub id: String,
    pub shape: ShapeConfig,
    pub spectrum: SpectrumConfig,
}
