use crate::{
    intersection::Intersection, ray::Ray, sampler::Sampler, spectrum::Spectrum, vector::Vector,
};

pub trait Object {
    fn reflectance(&self, wo: Vector, n: Vector, wi: Vector) -> Spectrum;
    fn probability(&self, wo: Vector, n: Vector, wi: Vector) -> f64;
    fn generate_ray(&self, n: Vector, wi: Vector, sampler: &dyn Sampler) -> Ray;
    fn intersect(&self, ray: Ray) -> Option<Intersection<'_>>;
    fn id(&self) -> u64;
}
