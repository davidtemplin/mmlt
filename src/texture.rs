use crate::spectrum::{Spectrum, SpectrumConfig};

use serde::{Deserialize, Serialize};

pub trait Texture {
    fn evaluate(&self) -> Spectrum;
}

pub struct ConstantTexture {
    value: Spectrum,
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
    Constant { spectrum: SpectrumConfig },
}
