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
    vector::Vector,
};

pub trait Light {
    fn radiance(&self, normal: Vector, direction: Vector) -> Spectrum;
    fn probability(&self, normal: Vector, direction: Vector) -> Option<f64>;
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
    fn radiance(&self, normal: Vector, direction: Vector) -> Spectrum {
        if normal.dot(direction) > 0.0 {
            self.radiance
        } else {
            Spectrum::black()
        }
    }

    fn probability(&self, normal: Vector, direction: Vector) -> Option<f64> {
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

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::{
        light::Light,
        shape::{Shape, Sphere},
        spectrum::{RgbSpectrum, Spectrum},
        vector::{Point, Vector},
    };

    use super::DiffuseAreaLight;

    #[test]
    fn test_diffuse_area_light_radiance() {
        let shape = Sphere::new(Point::new(0.0, 0.0, 0.0), 2.0);
        let radiance = RgbSpectrum::fill(10.0);
        let light = DiffuseAreaLight {
            id: String::from("light-1"),
            shape: Box::new(shape),
            radiance,
            light_count: 1,
        };
        let normal = Vector::new(0.0, 1.0, 0.0);
        let direction = Vector::new(1.0, 1.0, 0.0);
        assert_eq!(light.radiance(normal, direction), radiance);
        assert_eq!(light.radiance(normal, -direction), Spectrum::black());
    }

    #[test]
    fn test_diffuse_area_light_probability() {
        let light_count = 4;
        let radius = 2.0;
        let shape = Sphere::new(Point::new(0.0, 0.0, 0.0), radius);
        let area = shape.area();
        let radiance = RgbSpectrum::fill(10.0);
        let light = DiffuseAreaLight {
            id: String::from("light-1"),
            shape: Box::new(shape),
            radiance,
            light_count,
        };
        let normal = Vector::new(0.0, 1.0, 0.0);
        let direction = Vector::new(1.0, 1.0, 0.0);
        let p_light = 1.0 / light_count as f64;
        let p_point = 1.0 / area;
        let p_direction = normal.dot(direction.norm()) / PI;
        let p_total = p_light * p_point * p_direction;
        assert_eq!(light.probability(normal, direction), Some(p_total));
    }
}
