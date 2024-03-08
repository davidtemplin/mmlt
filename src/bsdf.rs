use std::{f64::consts::PI, fmt};

use crate::{sampler::Sampler, spectrum::Spectrum, util, vector::Vector};

#[derive(Debug)]
pub struct Bsdf {
    pub bxdfs: Vec<Box<dyn Bxdf>>,
}

pub trait Bxdf: fmt::Debug {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum;
    fn pdf(&self, wo: Vector, wi: Vector) -> Option<f64>;
    fn sample_direction(&self, wo: Vector, sampler: &mut dyn Sampler) -> Vector;
}

impl Bsdf {
    pub fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        self.bxdfs
            .iter()
            .map(|bxdf| bxdf.evaluate(wo, wi))
            .fold(Spectrum::black(), |a, b| a + b)
    }

    pub fn sample_direction(&self, wo: Vector, sampler: &mut dyn Sampler) -> Vector {
        let length = self.bxdfs.len() as f64;
        let r = sampler.sample(0.0..length).floor();
        let i = r as usize;
        self.bxdfs[i].sample_direction(wo, sampler)
    }

    pub fn pdf(&self, wo: Vector, wi: Vector) -> Option<f64> {
        let mut count = 0;
        let mut sum = 0.0;
        for bxdf in &self.bxdfs {
            let result = bxdf.pdf(wo, wi);
            if result.is_some() {
                count = count + 1;
            }
            let p = result.unwrap_or(0.0);
            sum = sum + p;
        }
        if count > 0 {
            let length = self.bxdfs.len() as f64;
            Some(sum / length)
        } else {
            None
        }
    }
}

#[derive(Debug)]
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
            Spectrum::black()
        }
    }

    fn pdf(&self, wo: Vector, wi: Vector) -> Option<f64> {
        let p = if util::same_hemisphere(self.normal, wo, wi) {
            util::abs_cos_theta(self.normal, wi) / PI
        } else {
            0.0
        };
        Some(p)
    }

    fn sample_direction(&self, wo: Vector, sampler: &mut dyn Sampler) -> Vector {
        let wi = util::cosine_sample_hemisphere(self.normal, sampler);
        if util::same_hemisphere(self.normal, wi, wo) {
            wi
        } else {
            -wi
        }
    }
}

#[derive(Debug)]
pub struct SpecularBrdf {
    scale: Spectrum,
    normal: Vector,
}

impl SpecularBrdf {
    pub fn new(normal: Vector, scale: Spectrum) -> SpecularBrdf {
        SpecularBrdf { scale, normal }
    }
}

impl Bxdf for SpecularBrdf {
    fn evaluate(&self, wo: Vector, wi: Vector) -> Spectrum {
        let d1 = wo.norm().dot(self.normal);
        let d2 = wi.norm().dot(self.normal);
        if util::equals(d1, d2, 0.0001) {
            self.scale
        } else {
            Spectrum::black()
        }
    }

    fn pdf(&self, _wo: Vector, _wi: Vector) -> Option<f64> {
        None
    }

    fn sample_direction(&self, wo: Vector, _sampler: &mut dyn Sampler) -> Vector {
        util::reflect(wo, self.normal)
    }
}

#[cfg(test)]
mod tests {
    use super::{Bxdf, DiffuseBrdf, SpecularBrdf};
    use crate::{bsdf::Bsdf, sampler::test::MockSampler, spectrum::Spectrum, util, vector::Vector};
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
    fn test_diffuse_brdf_pdf_same_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let actual = brdf.pdf(wo, wi);
        let expected = Some(util::abs_cos_theta(normal, wi) / PI);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_pdf_different_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, -1.0, 0.0);
        let actual = brdf.pdf(wo, wi);
        let expected = Some(0.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_sample_direction_parallel() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let wo = Vector::new(1.0, 1.0, 1.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        sampler.add(0.25);
        sampler.add(0.25);
        let direction = brdf.sample_direction(wo, &mut sampler);
        assert!(normal.dot(direction).is_sign_positive());
    }

    #[test]
    fn test_diffuse_brdf_sample_direction_non_parallel() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(1.0, 1.0, 1.0);
        let wo = Vector::new(2.0, 1.0, 1.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        sampler.add(0.25);
        sampler.add(0.25);
        let direction = brdf.sample_direction(wo, &mut sampler);
        assert!(normal.dot(direction).is_sign_positive());
    }

    #[test]
    fn test_specular_brdf_evaluate_exact() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let actual = brdf.evaluate(wo, wi);
        assert_eq!(actual, scale);
    }

    #[test]
    fn test_specular_brdf_evaluate_inexact() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.1, 0.0);
        let actual = brdf.evaluate(wo, wi);
        assert_eq!(actual, Spectrum::black());
    }

    #[test]
    fn test_specular_brdf_pdf() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let actual = brdf.pdf(wo, wi);
        assert_eq!(actual, None);
    }

    #[test]
    fn test_specular_brdf_sample_direction() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        let direction = brdf.sample_direction(wo, &mut sampler);
        let expected = util::reflect(wo, normal);
        assert_eq!(direction, expected);
    }

    #[test]
    fn test_bsdf_evaluate() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let actual = bsdf.evaluate(wo, wi);
        let expected = scale + (scale / PI);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bsdf_pdf() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let wi = Vector::new(-1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let actual = bsdf.pdf(wo, wi);
        let expected = Some((util::abs_cos_theta(normal, wi) / PI) / 2.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bsdf_sample_direction() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector::new(1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let mut sampler = MockSampler::new();
        sampler.add(0.9);
        let actual = bsdf.sample_direction(wo, &mut sampler);
        let expected = util::reflect(wo, normal);
        assert_eq!(actual, expected);
    }
}
