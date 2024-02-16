use crate::{intersection::Intersection, ray::Ray, sampler::Sampler, vector::Vector};

pub trait Shape {
    fn probability(&self, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
}
