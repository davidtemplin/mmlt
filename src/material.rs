use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    bsdf::{Bsdf, DielectricBxdf, DiffuseBrdf, SpecularBrdf},
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
pub struct MirrorMaterial {
    texture: Box<dyn Texture>,
}

impl MirrorMaterial {
    pub fn configure(config: &MirrorMaterialConfig) -> MirrorMaterial {
        MirrorMaterial {
            texture: config.texture.configure(),
        }
    }
}

impl Material for MirrorMaterial {
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

#[derive(Debug)]
pub struct DielectricMaterial {
    texture: Box<dyn Texture>,
    eta: f64,
}

impl DielectricMaterial {
    pub fn configure(config: &DielectricMaterialConfig) -> DielectricMaterial {
        DielectricMaterial {
            texture: config.texture.configure(),
            eta: config.eta,
        }
    }
}

impl Material for DielectricMaterial {
    fn compute_bsdf(&self, geometry: Geometry) -> Bsdf {
        Bsdf {
            bxdfs: vec![Box::new(DielectricBxdf::new(
                geometry.normal,
                self.texture.evaluate(geometry),
                self.eta,
            ))],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialConfig {
    Matte(MatteMaterialConfig),
    Glossy(GlossyMaterialConfig),
    Mirror(MirrorMaterialConfig),
    Dielectric(DielectricMaterialConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatteMaterialConfig {
    texture: TextureConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MirrorMaterialConfig {
    texture: TextureConfig,
}

impl MaterialConfig {
    pub fn configure(&self) -> Box<dyn Material> {
        match self {
            MaterialConfig::Matte(c) => Box::new(MatteMaterial::configure(&c)),
            MaterialConfig::Glossy(c) => Box::new(GlossyMaterial::configure(&c)),
            MaterialConfig::Mirror(c) => Box::new(MirrorMaterial::configure(&c)),
            MaterialConfig::Dielectric(c) => Box::new(DielectricMaterial::configure(&c)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GlossyMaterialConfig {
    diffuse_texture: TextureConfig,
    specular_texture: TextureConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DielectricMaterialConfig {
    texture: TextureConfig,
    eta: f64,
}
