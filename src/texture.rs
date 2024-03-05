use core::fmt;

use crate::{
    geometry::Geometry,
    spectrum::{Spectrum, SpectrumConfig},
};

use serde::{Deserialize, Serialize};

pub trait Texture: fmt::Debug {
    fn evaluate(&self, geometry: Geometry) -> Spectrum;
}

#[derive(Debug)]
pub struct ConstantTexture {
    value: Spectrum,
}

impl ConstantTexture {
    pub fn configure(config: &ConstantTextureConfig) -> ConstantTexture {
        ConstantTexture::new(Spectrum::configure(&config.spectrum))
    }

    pub fn new(value: Spectrum) -> ConstantTexture {
        ConstantTexture { value }
    }
}

impl Texture for ConstantTexture {
    fn evaluate(&self, _geometry: Geometry) -> Spectrum {
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

#[cfg(test)]
mod tests {
    use crate::{
        geometry::Geometry,
        spectrum::{Spectrum, SpectrumConfig},
        texture::Texture,
        vector::{Point, Vector},
    };

    use super::{ConstantTexture, ConstantTextureConfig};

    #[test]
    fn test_constant_texture_configure() {
        let config = ConstantTextureConfig {
            spectrum: SpectrumConfig {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
        };
        let texture = ConstantTexture::configure(&config);
        assert_eq!(texture.value, Spectrum::fill(1.0));
    }

    #[test]
    fn test_constant_texture_new() {
        let spectrum = Spectrum::fill(1.0);
        let texture = ConstantTexture::new(spectrum);
        assert_eq!(texture.value, spectrum);
    }

    #[test]
    fn test_constant_texture_evaluate() {
        let spectrum = Spectrum::fill(1.0);
        let texture = ConstantTexture::new(spectrum);
        let geometry = Geometry {
            point: Point::new(0.0, 0.0, 0.0),
            normal: Vector::new(0.0, 0.0, 0.0),
            direction: Vector::new(0.0, 0.0, 0.0),
        };
        assert_eq!(texture.evaluate(geometry), spectrum);
    }
}
