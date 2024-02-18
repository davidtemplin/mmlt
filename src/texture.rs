use crate::spectrum::Spectrum;

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
