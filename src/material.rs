use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    bsdf::{Bsdf, DiffuseBrdf, SpecularBrdf},
    geometry::Geometry,
    texture::{Texture, TextureConfig},
};

pub trait Material: fmt::Debug {
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf;
}

#[derive(Debug)]
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
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        Bsdf {
            bxdfs: vec![Box::new(DiffuseBrdf::new(
                geometry.normal,
                self.texture.evaluate(geometry),
            ))],
        }
    }
}

#[derive(Debug)]
pub struct MetalMaterial {
    texture: Box<dyn Texture>,
}

impl MetalMaterial {
    pub fn configure(config: &MetalMaterialConfig) -> MetalMaterial {
        MetalMaterial {
            texture: config.texture.configure(),
        }
    }
}

impl Material for MetalMaterial {
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        Bsdf {
            bxdfs: vec![Box::new(SpecularBrdf::new(
                geometry.normal,
                self.texture.evaluate(geometry),
            ))],
        }
    }
}

#[derive(Debug)]
pub struct GlossyMaterial {
    diffuse_texture: Box<dyn Texture>,
    specular_texture: Box<dyn Texture>,
}

impl GlossyMaterial {
    pub fn configure(config: &GlossyMaterialConfig) -> GlossyMaterial {
        GlossyMaterial {
            diffuse_texture: config.diffuse_texture.configure(),
            specular_texture: config.specular_texture.configure(),
        }
    }
}

impl Material for GlossyMaterial {
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        Bsdf {
            bxdfs: vec![
                Box::new(DiffuseBrdf::new(
                    geometry.normal,
                    self.diffuse_texture.evaluate(geometry),
                )),
                Box::new(SpecularBrdf::new(
                    geometry.normal,
                    self.specular_texture.evaluate(geometry),
                )),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialConfig {
    Matte(MatteMaterialConfig),
    Glossy(GlossyMaterialConfig),
    Metal(MetalMaterialConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatteMaterialConfig {
    texture: TextureConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetalMaterialConfig {
    texture: TextureConfig,
}

impl MaterialConfig {
    pub fn configure(&self) -> Box<dyn Material> {
        match self {
            MaterialConfig::Matte(c) => Box::new(MatteMaterial::configure(&c)),
            MaterialConfig::Glossy(c) => Box::new(GlossyMaterial::configure(&c)),
            MaterialConfig::Metal(c) => Box::new(MetalMaterial::configure(&c)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GlossyMaterialConfig {
    diffuse_texture: TextureConfig,
    specular_texture: TextureConfig,
}
