use std::{cell::OnceCell, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    bsdf::Bsdf,
    geometry::Geometry,
    interaction::{Interaction, ObjectInteraction},
    material::{Material, MaterialConfig},
    ray::Ray,
    shape::{Shape, ShapeConfig},
};

pub trait Object: fmt::Debug {
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf;
    fn id(&self) -> &String;
}

#[derive(Debug)]
pub struct GeometricObject {
    id: String,
    shape: Box<dyn Shape>,
    material: Box<dyn Material>,
}

impl Object for GeometricObject {
    fn intersect(&self, ray: Ray) -> Option<Interaction> {
        let geometry = self.shape.intersect(ray)?;
        let interaction = ObjectInteraction {
            object: self,
            geometry,
            bsdf: OnceCell::new(),
        };
        Some(Interaction::Object(interaction))
    }

    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        self.material.compute_bsdf(geometry)
    }

    fn id(&self) -> &String {
        &self.id
    }
}

impl GeometricObject {
    pub fn configure(config: &GeometricObjectConfig) -> GeometricObject {
        GeometricObject {
            id: config.id.clone(),
            shape: config.shape.configure(),
            material: config.material.configure(),
        }
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
