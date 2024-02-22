use serde::{Deserialize, Serialize};

use crate::{
    bsdf::{Bsdf, DiffuseBrdf},
    geometry::Geometry,
    texture::{Texture, TextureConfig},
};

pub trait Material {
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf;
}

pub struct MatteMaterial {
    texture: Box<dyn Texture>,
}

impl MatteMaterial {
    pub fn configure(config: &MatteMaterialConfig) -> MatteMaterial {
        MatteMaterial {
            texture: config.texture.configure(),
        }
    }
}

impl Material for MatteMaterial {
    fn compute_bsdf(&self, _geometry: Geometry) -> Bsdf {
        Bsdf {
            bxdfs: vec![Box::new(DiffuseBrdf::new(self.texture.evaluate()))],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialConfig {
    Matte(MatteMaterialConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatteMaterialConfig {
    texture: TextureConfig,
}

impl MaterialConfig {
    pub fn configure(&self) -> Box<dyn Material> {
        match self {
            MaterialConfig::Matte(c) => Box::new(MatteMaterial::configure(&c)),
        }
    }
}
