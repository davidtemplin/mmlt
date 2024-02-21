use serde::{Deserialize, Serialize};

use crate::{
    bsdf::{Bsdf, DiffuseBrdf},
    texture::{Texture, TextureConfig},
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialConfig {
    Matte { texture: TextureConfig },
}
