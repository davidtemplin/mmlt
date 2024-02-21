use serde::{Deserialize, Serialize};

use crate::{
    intersection::Intersection, material::MaterialConfig, ray::Ray, sampler::Sampler,
    shape::ShapeConfig, spectrum::Spectrum, vector::Vector,
};

pub trait Object {
    fn reflectance(&self, wo: Vector, n: Vector, wi: Vector) -> Spectrum;
    fn probability(&self, wo: Vector, n: Vector, wi: Vector) -> f64;
    fn generate_ray(&self, n: Vector, wi: Vector, sampler: &dyn Sampler) -> Ray;
    fn intersect(&self, ray: Ray) -> Option<Intersection<'_>>;
    fn id(&self) -> u64;
}

pub struct GeometricObject {}

impl Object for GeometricObject {
    fn reflectance(&self, wo: Vector, n: Vector, wi: Vector) -> Spectrum {
        todo!()
    }

    fn probability(&self, wo: Vector, n: Vector, wi: Vector) -> f64 {
        todo!()
    }

    fn generate_ray(&self, n: Vector, wi: Vector, sampler: &dyn Sampler) -> Ray {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection<'_>> {
        todo!()
    }

    fn id(&self) -> u64 {
        todo!()
    }
}

impl GeometricObject {
    pub fn configure(config: &GeometricObjectConfig) -> GeometricObject {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ObjectConfig {
    Geometric(GeometricObjectConfig),
}

impl ObjectConfig {
    pub fn configure(&self) -> Box<dyn Object> {
        match self {
            ObjectConfig::Geometric(config) => Box::new(GeometricObject::configure(config)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeometricObjectConfig {
    id: String,
    shape: ShapeConfig,
    material: MaterialConfig,
}
