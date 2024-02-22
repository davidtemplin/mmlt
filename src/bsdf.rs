use std::f64::consts::PI;

use crate::{sampler::Sampler, spectrum::Spectrum, vector::Vector};

pub struct Bsdf {
    pub bxdfs: Vec<Box<dyn Bxdf>>,
}

pub trait Bxdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum;
    fn probability(&self, wo: Vector, wi: Vector) -> f64;
    fn sample_direction(&self, sampler: &mut dyn Sampler) -> Vector;
}

impl Bsdf {
    pub fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        self.bxdfs
            .iter()
            .map(|bxdf| bxdf.evaluate(wo, wi))
            .fold(Spectrum::black(), |a, b| a + b)
    }

    pub fn sample_direction(&self, sampler: &mut dyn Sampler) -> Vector {
        let r = sampler.sample(0.0..1.0);
        let i = (r * self.bxdfs.len() as f64).floor() as usize;
        self.bxdfs[i].sample_direction(sampler)
    }

    pub fn probability(&self, wo: Vector, wi: Vector) -> f64 {
        self.bxdfs
            .iter()
            .map(|bxdf| bxdf.probability(wo, wi))
            .fold(0.0, |a, b| a + b)
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

    fn sample_direction(&self, sampler: &mut dyn Sampler) -> Vector {
        todo!()
    }
}
