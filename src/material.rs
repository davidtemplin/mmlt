use crate::{
    bsdf::{Bsdf, DiffuseBrdf},
    texture::Texture,
};

pub trait Material {
    fn compute_bsdf(&self) -> Bsdf;
}

pub struct MatteMaterial {
    texture: Box<dyn Texture>,
}

impl MatteMaterial {
    fn compute_bsdf(&self) -> Bsdf {
        Bsdf {
            bxdfs: vec![Box::new(DiffuseBrdf::new(self.texture.evaluate()))],
        }
    }
}
