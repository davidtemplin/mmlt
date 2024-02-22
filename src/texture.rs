use crate::spectrum::{Spectrum, SpectrumConfig};

use serde::{Deserialize, Serialize};

pub trait Texture {
    fn evaluate(&self) -> Spectrum;
}

pub struct ConstantTexture {
    value: Spectrum,
}

impl ConstantTexture {
    pub fn configure(config: &ConstantTextureConfig) -> ConstantTexture {
        ConstantTexture {
            value: Spectrum::configure(&config.spectrum),
        }
    }
}

impl Texture for ConstantTexture {
    fn evaluate(&self) -> Spectrum {
        self.value
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum TextureConfig {
    Constant(ConstantTextureConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstantTextureConfig {
    spectrum: SpectrumConfig,
}

impl TextureConfig {
    pub fn configure(&self) -> Box<dyn Texture> {
        match self {
            TextureConfig::Constant(c) => Box::new(ConstantTexture::configure(&c)),
        }
    }
}
