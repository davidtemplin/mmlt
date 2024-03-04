use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    interaction::{Interaction, LightInteraction},
    ray::Ray,
    sampler::Sampler,
    shape::{Shape, ShapeConfig},
    spectrum::{Spectrum, SpectrumConfig},
    util,
    vector::{Point, Vector},
};

pub trait Light {
    fn radiance(&self, point: Point, direction: Vector) -> Spectrum;
    fn probability(&self, point: Point, normal: Vector, direction: Vector) -> Option<f64>;
    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction;
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn id(&self) -> &String;
}

pub struct DiffuseAreaLight {
    id: String,
    shape: Box<dyn Shape>,
    radiance: Spectrum,
    light_count: usize,
}

impl Light for DiffuseAreaLight {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum {
        if normal.dot(direction) > 0.0 {
            self.radiance
        } else {
            Spectrum::black()
        }
    }

    fn probability(&self, _point: Point, normal: Vector, direction: Vector) -> Option<f64> {
        let p = direction.norm().dot(normal) / (self.light_count as f64 * self.shape.area() * PI);
        Some(p)
    }

    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction {
        let geometry = self.shape.sample_geometry(sampler);

        let direction = util::cosine_sample_hemisphere(geometry.normal, sampler);

        let light_interaction = LightInteraction {
            light: self,
            geometry: Geometry {
                point: geometry.point,
                direction,
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
    pub fn configure(config: &DiffuseAreaLightConfig, light_count: usize) -> DiffuseAreaLight {
        DiffuseAreaLight {
            id: config.id.clone(),
            shape: config.shape.configure(),
            radiance: Spectrum::configure(&config.spectrum),
            light_count,
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
    pub fn configure(&self, light_count: usize) -> Box<dyn Light> {
        match self {
            LightConfig::DiffuseArea(config) => {
                Box::new(DiffuseAreaLight::configure(config, light_count))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiffuseAreaLightConfig {
    pub id: String,
    pub shape: ShapeConfig,
    pub spectrum: SpectrumConfig,
}
