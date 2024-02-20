use std::f64::consts::PI;

use crate::{spectrum::Spectrum, vector::Vector};

pub struct Bsdf {
    pub bxdfs: Vec<Box<dyn Bxdf>>,
}

pub trait Bxdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum;
    fn probability(&self, wo: Vector, wi: Vector) -> f64;
}

impl Bsdf {
    pub fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        self.bxdfs
            .iter()
            .map(|bxdf| bxdf.evaluate(wo, wi))
            .fold(Spectrum::black(), |a, b| a + b)
    }
}

pub struct DiffuseBrdf {
    scale: Spectrum,
}

impl DiffuseBrdf {
    pub fn new(scale: Spectrum) -> DiffuseBrdf {
        DiffuseBrdf { scale }
    }
}

impl Bxdf for DiffuseBrdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        self.scale * (1.0 / PI)
    }

    fn probability(&self, _wo: Vector, _wi: Vector) -> f64 {
        1.0 / (2.0 * PI)
    }
}
