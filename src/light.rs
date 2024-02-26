use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    interaction::{Interaction, LightInteraction},
    ray::Ray,
    sampler::Sampler,
    shape::{Shape, ShapeConfig},
    spectrum::{Spectrum, SpectrumConfig},
    vector::Vector,
};

pub trait Light {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum;
    fn probability(&self, direction: Vector) -> Option<f64>;
    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction;
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn id(&self) -> &String;
}

pub struct DiffuseAreaLight {
    id: String,
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

    fn probability(&self, _direction: Vector) -> Option<f64> {
        let p = 1.0 / self.shape.area();
        Some(p)
    }

    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction {
        let geometry = self.shape.sample_intersection(sampler);
        let light_interaction = LightInteraction {
            light: self,
            geometry: Geometry {
                point: geometry.point,
                direction: geometry.direction,
                normal: geometry.normal,
            },
        };
        Interaction::Light(light_interaction)
    }

    fn intersect(&self, ray: Ray) -> Option<Interaction> {
        let geometry = self.shape.intersect(ray)?;
        let light_interaction = LightInteraction {
            light: self,
            geometry: Geometry {
                point: geometry.point,
                direction: geometry.direction,
                normal: geometry.normal,
            },
        };
        let interaction = Interaction::Light(light_interaction);
        Some(interaction)
    }

    fn id(&self) -> &String {
        &self.id
    }
}

impl DiffuseAreaLight {
    pub fn configure(config: &DiffuseAreaLightConfig) -> DiffuseAreaLight {
        DiffuseAreaLight {
            id: config.id.clone(),
            shape: config.shape.configure(),
            radiance: Spectrum::configure(&config.spectrum),
        }
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
