use crate::{
    intersection::Intersection, ray::Ray, sampler::Sampler, shape::Shape, spectrum::Spectrum,
    vector::Vector,
};

pub trait Light {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum;
    fn probability(&self, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection<'_>>;
    fn id(&self) -> u64;
}

pub struct DiffuseAreaLight {
    id: u64,
    shape: Box<dyn Shape>,
    radiance: Spectrum,
}

impl Light for DiffuseAreaLight {
    fn radiance(&self, direction: Vector, normal: Vector) -> Spectrum {
        if normal.dot(direction) > 0.0 {
            self.radiance
        } else {
            Spectrum::black()
        }
    }

    fn probability(&self, direction: Vector) -> f64 {
        self.shape.probability(direction)
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        self.shape.sample_intersection(sampler)
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        self.shape.intersect(ray)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
