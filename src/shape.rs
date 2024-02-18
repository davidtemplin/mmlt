use crate::{
    intersection::Intersection,
    ray::Ray,
    sampler::Sampler,
    vector::{Point, Vector},
};

pub trait Shape {
    fn probability(&self, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
}

pub struct Sphere {
    center: Point,
    radius: Vector,
}

impl Shape for Sphere {
    fn probability(&self, direction: Vector) -> f64 {
        todo!()
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        todo!()
    }
}

pub struct Rectangle {
    min: Point,
    max: Point,
}

impl Shape for Rectangle {
    fn probability(&self, direction: Vector) -> f64 {
        todo!()
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        todo!()
    }
}
