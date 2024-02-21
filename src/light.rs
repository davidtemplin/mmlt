use serde::{Deserialize, Serialize};

use crate::{
    interaction::Interaction,
    ray::Ray,
    sampler::Sampler,
    shape::{Shape, ShapeConfig},
    spectrum::{Spectrum, SpectrumConfig},
    vector::Vector,
};

pub trait Light {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum;
    fn probability(&self, direction: Vector) -> f64;
    fn sample_interaction(&self, sampler: &dyn Sampler) -> Interaction;
    fn intersect(&self, ray: Ray) -> Option<Interaction<'_>>;
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

    fn sample_interaction(&self, sampler: &dyn Sampler) -> Interaction {
        self.shape.sample_interaction(sampler)
    }

    fn intersect(&self, ray: Ray) -> Option<Interaction> {
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
