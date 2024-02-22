use serde::{Deserialize, Serialize};

use crate::{
    bsdf::Bsdf,
    geometry::Geometry,
    interaction::Interaction,
    material::{Material, MaterialConfig},
    ray::Ray,
    shape::{Shape, ShapeConfig},
};

pub trait Object {
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf;
    fn id(&self) -> u64;
}

pub struct GeometricObject {
    shape: Box<dyn Shape>,
    material: Box<dyn Material>,
}

impl Object for GeometricObject {
    fn intersect(&self, ray: Ray) -> Option<Interaction> {
        todo!()
    }

    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        self.material.compute_bsdf(geometry)
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
