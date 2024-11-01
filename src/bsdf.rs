use std::{f64::consts::PI, fmt};

use crate::{
    approx::ApproxEq,
    sampler::Sampler,
    spectrum::Spectrum,
    types::PathType,
    util::{self},
    vector::Vector3,
};

#[derive(Debug)]
pub struct Bsdf {
    pub bxdfs: Vec<Box<dyn Bxdf>>,
}

pub trait Bxdf: fmt::Debug {
    fn evaluate(&self, wo: Vector3, wi: Vector3, context: EvaluationContext) -> Spectrum;
    fn sampling_pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64>;
    fn pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64>;
    fn sample_direction(
        &self,
        wx: Vector3,
        path_type: PathType,
        sampler: &mut dyn Sampler,
    ) -> Option<Vector3>;
}

#[derive(Debug, Copy, Clone)]
pub struct EvaluationContext {
    pub geometry_term: f64,
    pub path_type: PathType,
}

impl Bsdf {
    pub fn evaluate(&self, wo: Vector3, wi: Vector3, context: EvaluationContext) -> Spectrum {
        self.bxdfs
            .iter()
            .map(|bxdf| bxdf.evaluate(wo, wi, context))
            .fold(Spectrum::black(), |a, b| a + b)
    }

    pub fn sample_direction(
        &self,
        wx: Vector3,
        path_type: PathType,
        sampler: &mut dyn Sampler,
    ) -> Option<Vector3> {
        let length = self.bxdfs.len() as f64;
        let r = sampler.sample(0.0..length).floor();
        let i = r as usize;
        self.bxdfs[i].sample_direction(wx, path_type, sampler)
    }

    pub fn sampling_pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64> {
        let mut count = 0;
        let mut sum = 0.0;
        for bxdf in &self.bxdfs {
            let result = bxdf.sampling_pdf(wo, wi, path_type);
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

    pub fn pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64> {
        let mut count = 0;
        let mut sum = 0.0;
        for bxdf in &self.bxdfs {
            let result = bxdf.pdf(wo, wi, path_type);
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
    normal: Vector3,
}

impl DiffuseBrdf {
    pub fn new(normal: Vector3, scale: Spectrum) -> DiffuseBrdf {
        DiffuseBrdf { normal, scale }
    }
}

impl Bxdf for DiffuseBrdf {
    fn evaluate(&self, wo: Vector3, wi: Vector3, _: EvaluationContext) -> Spectrum {
        if util::same_hemisphere(self.normal, wo, wi) {
            self.scale / PI
        } else {
            Spectrum::black()
        }
    }

    fn sampling_pdf(&self, _: Vector3, _: Vector3, _: PathType) -> Option<f64> {
        None
    }

    fn pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64> {
        let p = if util::same_hemisphere(self.normal, wo, wi) {
            match path_type {
                PathType::Camera => util::abs_cos_theta(self.normal, wi) / PI,
                PathType::Light => util::abs_cos_theta(self.normal, wo) / PI,
            }
        } else {
            0.0
        };
        Some(p)
    }

    fn sample_direction(
        &self,
        wo: Vector3,
        _: PathType,
        sampler: &mut dyn Sampler,
    ) -> Option<Vector3> {
        let wi = util::cosine_sample_hemisphere(self.normal, sampler);
        if util::same_hemisphere(self.normal, wi, wo) {
            Some(wi)
        } else {
            Some(-wi)
        }
    }
}

#[derive(Debug)]
pub struct SpecularBrdf {
    scale: Spectrum,
    normal: Vector3,
}

impl SpecularBrdf {
    pub fn new(normal: Vector3, scale: Spectrum) -> SpecularBrdf {
        SpecularBrdf { scale, normal }
    }
}

impl Bxdf for SpecularBrdf {
    fn evaluate(&self, wo: Vector3, wi: Vector3, context: EvaluationContext) -> Spectrum {
        let d1 = wo.norm().dot(self.normal);
        let d2 = wi.norm().dot(self.normal);
        if util::equals(d1, d2, 0.0001) {
            self.scale / context.geometry_term
        } else {
            Spectrum::black()
        }
    }

    fn sampling_pdf(&self, _: Vector3, _: Vector3, _: PathType) -> Option<f64> {
        None
    }

    fn pdf(&self, _: Vector3, _: Vector3, _: PathType) -> Option<f64> {
        None
    }

    fn sample_direction(&self, wx: Vector3, _: PathType, _: &mut dyn Sampler) -> Option<Vector3> {
        Some(util::reflect(wx, self.normal))
    }
}

#[derive(Debug)]
pub struct DielectricBxdf {
    scale: Spectrum,
    normal: Vector3,
    eta: f64,
}

impl DielectricBxdf {
    pub fn new(normal: Vector3, scale: Spectrum, eta: f64) -> DielectricBxdf {
        DielectricBxdf { normal, scale, eta }
    }

    fn evaluate_internal(&self, wi: Vector3, wt: Vector3, adjoint: bool) -> Spectrum {
        let reflection = util::reflect(wi.norm(), self.normal);
        if wt.norm().approx_eq(reflection, 1e-6) {
            let cos_theta = util::cos_theta(self.normal, wi);
            let r = util::fresnel_dielectric(cos_theta, self.eta);
            self.scale * r
        } else {
            let refraction = util::refract(wi.norm(), self.normal.norm(), self.eta);
            if refraction.is_none() {
                return Spectrum::black();
            }
            if wt.norm().approx_eq(refraction.unwrap(), 1e-6) {
                let cos_theta = util::cos_theta(self.normal, wi);
                let r = util::fresnel_dielectric(cos_theta, self.eta);
                let t = 1.0 - r;
                let eta_actual = if cos_theta < 0.0 {
                    1.0 / self.eta
                } else {
                    self.eta
                };
                let adjoint_factor = if adjoint { util::sqr(eta_actual) } else { 1.0 };
                self.scale * t / adjoint_factor
            } else {
                Spectrum::black()
            }
        }
    }

    fn sampling_pdf_internal(&self, wi: Vector3, wt: Vector3) -> Option<f64> {
        let reflection = util::reflect(wi.norm(), self.normal);
        if wt.norm().approx_eq(reflection, 1e-6) {
            let cos_theta = util::cos_theta(self.normal, wi);
            let r = util::fresnel_dielectric(cos_theta, self.eta);
            Some(r)
        } else {
            let refraction = util::refract(wi.norm(), self.normal.norm(), self.eta);
            if refraction.is_none() {
                return None;
            }
            if wt.norm().approx_eq(refraction.unwrap(), 1e-6) {
                let cos_theta = util::cos_theta(self.normal, wi);
                let r = util::fresnel_dielectric(cos_theta, self.eta);
                let t = 1.0 - r;
                Some(t)
            } else {
                None
            }
        }
    }
}

impl Bxdf for DielectricBxdf {
    fn evaluate(&self, wo: Vector3, wi: Vector3, context: EvaluationContext) -> Spectrum {
        let result = match context.path_type {
            PathType::Camera => self.evaluate_internal(wo, wi, true),
            PathType::Light => self.evaluate_internal(wi, wo, false),
        };
        result / context.geometry_term
    }

    fn sampling_pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64> {
        match path_type {
            PathType::Camera => self.sampling_pdf_internal(wo, wi),
            PathType::Light => self.sampling_pdf_internal(wi, wo),
        }
    }

    fn pdf(&self, _: Vector3, _: Vector3, _: PathType) -> Option<f64> {
        None
    }

    fn sample_direction(
        &self,
        wx: Vector3,
        _: PathType,
        sampler: &mut dyn Sampler,
    ) -> Option<Vector3> {
        // TODO: disable reflection when internal to object; use flags?
        let cos_theta_i = util::cos_theta(self.normal, wx);
        let r = util::fresnel_dielectric(cos_theta_i, self.eta);
        if sampler.sample(0.0..1.0) < r {
            Some(util::reflect(wx, self.normal))
        } else {
            util::refract(wx.norm(), self.normal.norm(), self.eta)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bxdf, DielectricBxdf, DiffuseBrdf, SpecularBrdf};
    use crate::{
        approx::ApproxEq,
        bsdf::{Bsdf, EvaluationContext},
        sampler::test::MockSampler,
        spectrum::Spectrum,
        types::PathType,
        util,
        vector::Vector3,
    };
    use std::f64::consts::PI;

    #[test]
    fn test_diffuse_brdf_evaluate_same_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let context = EvaluationContext {
            geometry_term: 1.0,
            path_type: PathType::Camera,
        };
        let actual = brdf.evaluate(wo, wi, context);
        let expected = scale / PI;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_evaluate_different_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, -1.0, 0.0);
        let context = EvaluationContext {
            geometry_term: 1.0,
            path_type: PathType::Camera,
        };
        let actual = brdf.evaluate(wo, wi, context);
        let expected = Spectrum::fill(0.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_pdf_same_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let actual = brdf.pdf(wo, wi, PathType::Camera);
        let expected = Some(util::abs_cos_theta(normal, wi) / PI);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_pdf_different_hemisphere() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, -1.0, 0.0);
        let actual = brdf.pdf(wo, wi, PathType::Camera);
        let expected = Some(0.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diffuse_brdf_sample_direction_parallel() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let wo = Vector3::new(1.0, 1.0, 1.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        sampler.add(0.25);
        sampler.add(0.25);
        let direction = brdf
            .sample_direction(wo, PathType::Camera, &mut sampler)
            .unwrap();
        assert!(normal.dot(direction).is_sign_positive());
    }

    #[test]
    fn test_diffuse_brdf_sample_direction_non_parallel() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(1.0, 1.0, 1.0);
        let wo = Vector3::new(2.0, 1.0, 1.0);
        let brdf = DiffuseBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        sampler.add(0.25);
        sampler.add(0.25);
        let direction = brdf
            .sample_direction(wo, PathType::Camera, &mut sampler)
            .unwrap();
        assert!(normal.dot(direction).is_sign_positive());
    }

    #[test]
    fn test_specular_brdf_evaluate_exact() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let context = EvaluationContext {
            geometry_term: 1.0,
            path_type: PathType::Camera,
        };
        let actual = brdf.evaluate(wo, wi, context);
        assert_eq!(actual, scale);
    }

    #[test]
    fn test_specular_brdf_evaluate_inexact() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.1, 0.0);
        let context = EvaluationContext {
            geometry_term: 1.0,
            path_type: PathType::Camera,
        };
        let actual = brdf.evaluate(wo, wi, context);
        assert_eq!(actual, Spectrum::black());
    }

    #[test]
    fn test_specular_brdf_pdf() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let actual = brdf.pdf(wo, wi, PathType::Camera);
        assert_eq!(actual, None);
    }

    #[test]
    fn test_specular_brdf_sample_direction() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let brdf = SpecularBrdf::new(normal, scale);
        let mut sampler = MockSampler::new();
        let direction = brdf
            .sample_direction(wo, PathType::Camera, &mut sampler)
            .unwrap();
        let expected = util::reflect(wo, normal);
        assert_eq!(direction, expected);
    }

    #[test]
    fn test_dielectric_bxdf() {
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let scale = Spectrum::fill(1.0);
        let eta = 1.6;
        let theta_i = 30.0 * PI / 180.0;
        let wi = Vector3::new(-f64::sin(theta_i), f64::cos(theta_i), 0.0);
        let theta_t = 18.20996 * PI / 180.0;
        let mut expected_wt = Vector3::new(f64::sin(theta_t), -f64::cos(theta_t), 0.0);
        let bxdf = DielectricBxdf::new(normal, scale, eta);
        let mut sampler = MockSampler::new();

        // Camera path
        let mut path_type = PathType::Camera;

        // Refraction
        sampler.add(0.5); // 0.5 > r
        let mut wt = bxdf.sample_direction(wi, path_type, &mut sampler).unwrap();
        assert!(wt.approx_eq(expected_wt, 1e-5));
        let mut pdf = bxdf.sampling_pdf(wi, wt, path_type).unwrap();
        let r = 0.0549528214871777;
        assert!(util::equals(pdf, 1.0 - r, 1e-5));
        let geometry_term = 0.4; // arbitrary
        let mut context = EvaluationContext {
            geometry_term,
            path_type,
        };
        let mut e = bxdf.evaluate(wi, wt, context);
        let mut expected_e = Spectrum::fill(((1.0 - r) / geometry_term) / util::sqr(eta));
        assert!(e.approx_eq(expected_e, 1e-5));

        // Reflection
        sampler.add(0.04); // 0.04 < r
        wt = bxdf.sample_direction(wi, path_type, &mut sampler).unwrap();
        expected_wt = Vector3::new(-wi.x, wi.y, 0.0);
        assert!(wt.approx_eq(expected_wt, 1e-5));
        pdf = bxdf.sampling_pdf(wi, wt, path_type).unwrap();
        assert!(util::equals(pdf, r, 1e-5));
        e = bxdf.evaluate(wi, wt, context);
        expected_e = Spectrum::fill(r / geometry_term);
        assert!(e.approx_eq(expected_e, 1e-5));

        // Light path
        path_type = PathType::Light;
        context = EvaluationContext {
            geometry_term,
            path_type,
        };

        // Refraction
        sampler.add(0.5);
        wt = bxdf.sample_direction(wi, path_type, &mut sampler).unwrap();
        expected_wt = Vector3::new(f64::sin(theta_t), -f64::cos(theta_t), 0.0);
        assert!(wt.approx_eq(expected_wt, 1e-5));
        pdf = bxdf.sampling_pdf(wt, wi, path_type).unwrap();
        assert!(util::equals(pdf, 1.0 - r, 1e-5));
        e = bxdf.evaluate(wt, wi, context);
        expected_e = Spectrum::fill((1.0 - r) / geometry_term);
        assert!(e.approx_eq(expected_e, 1e-5));

        // Reflection
        sampler.add(0.04); // 0.04 < r
        wt = bxdf.sample_direction(wi, path_type, &mut sampler).unwrap();
        expected_wt = Vector3::new(-wi.x, wi.y, 0.0);
        assert!(wt.approx_eq(expected_wt, 1e-5));
        pdf = bxdf.sampling_pdf(wt, wi, path_type).unwrap();
        assert!(util::equals(pdf, r, 1e-5));
        e = bxdf.evaluate(wt, wi, context);
        expected_e = Spectrum::fill(r / geometry_term);
        assert!(e.approx_eq(expected_e, 1e-5));
    }

    #[test]
    fn test_bsdf_evaluate() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let context = EvaluationContext {
            geometry_term: 1.0,
            path_type: PathType::Camera,
        };
        let actual = bsdf.evaluate(wo, wi, context);
        let expected = scale + (scale / PI);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bsdf_pdf() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let wi = Vector3::new(-1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let actual = bsdf.pdf(wo, wi, PathType::Camera);
        let expected = Some((util::abs_cos_theta(normal, wi) / PI) / 2.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bsdf_sample_direction() {
        let scale = Spectrum::fill(0.8);
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let brdf1 = DiffuseBrdf::new(normal, scale);
        let brdf2 = SpecularBrdf::new(normal, scale);
        let wo = Vector3::new(1.0, 1.0, 0.0);
        let bsdf = Bsdf {
            bxdfs: vec![Box::new(brdf1), Box::new(brdf2)],
        };
        let mut sampler = MockSampler::new();
        sampler.add(0.9);
        let actual = bsdf
            .sample_direction(wo, PathType::Camera, &mut sampler)
            .unwrap();
        let expected = util::reflect(wo, normal);
        assert_eq!(actual, expected);
    }
}
