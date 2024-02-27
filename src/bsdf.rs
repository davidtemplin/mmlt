use std::f64::consts::PI;

use crate::{sampler::Sampler, spectrum::Spectrum, util, vector::Vector};

pub struct Bsdf {
    pub bxdfs: Vec<Box<dyn Bxdf>>,
}

pub trait Bxdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum;
    fn probability(&self, wo: Vector, wi: Vector) -> Option<f64>;
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

    pub fn probability(&self, wo: Vector, wi: Vector) -> Option<f64> {
        let p = self
            .bxdfs
            .iter()
            .map(|bxdf| bxdf.probability(wo, wi).unwrap_or(0.0))
            .fold(0.0, |a, b| a + b)
            / (self.bxdfs.len() as f64);
        Some(p)
    }
}

pub struct DiffuseBrdf {
    scale: Spectrum,
    normal: Vector,
}

impl DiffuseBrdf {
    pub fn new(normal: Vector, scale: Spectrum) -> DiffuseBrdf {
        DiffuseBrdf { normal, scale }
    }
}

impl Bxdf for DiffuseBrdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        if util::same_hemisphere(self.normal, wo, wi) {
            self.scale / PI
        } else {
            Spectrum::fill(0.0)
        }
    }

    fn probability(&self, wo: Vector, wi: Vector) -> Option<f64> {
        let p = if util::same_hemisphere(self.normal, wo, wi) {
            util::abs_cos_theta(self.normal, wi) / PI
        } else {
            0.0
        };
        Some(p)
    }

    fn sample_direction(&self, sampler: &mut dyn Sampler) -> Vector {
        util::cosine_sample_hemisphere(self.normal, sampler)
    }
}

#[cfg(test)]
mod tests {
    use super::{Bxdf, DiffuseBrdf};
    use crate::{sampler::test::MockSampler, spectrum::Spectrum, util, vector::Vector};
    use std::f64::consts::PI;

    #[test]
    fn test_diffuse_brdf_evaluate_same_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let actual = brdf.evaluate(wo, wi);
        let expected = scale / PI;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_evaluate_different_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, -1.0, 0.0);
        let actual = brdf.evaluate(wo, wi);
        let expected = Spectrum::fill(0.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_probability_same_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let actual = brdf.probability(wo, wi);
        let expected = Some(util::abs_cos_theta(normal, wi) / PI);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_probability_different_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, -1.0, 0.0);
        let actual = brdf.probability(wo, wi);
        let expected = Some(0.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_sample_direction() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        sampler.add(0.25);
        sampler.add(0.25);
        let direction = brdf.sample_direction(&mut sampler);
        assert!(normal.dot(direction).is_sign_positive());
    }
}
