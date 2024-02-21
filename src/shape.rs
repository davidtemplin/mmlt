use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    intersection::{Intersection, ObjectIntersection, Orientation},
    ray::Ray,
    sampler::Sampler,
    vector::{Point, PointConfig, Vector},
};

pub trait Shape {
    fn probability(&self, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
}

pub struct Sphere {
    center: Point,
    radius: f64,
}

impl Shape for Sphere {
    fn probability(&self, direction: Vector) -> f64 {
        1.0 / (4.0 * PI * self.radius * self.radius)
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        todo!()
    }
}

pub struct Rectangle {
    origin: Point,
    left: Point,
    right: Point,
}

impl Shape for Rectangle {
    fn probability(&self, _direction: Vector) -> f64 {
        let left_length = (self.left - self.origin).len();
        let right_length = (self.right - self.origin).len();
        let area = left_length * right_length;
        1.0 / area
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Intersection {
        todo!()
    }

    // TODO: this cannot compute an intersection; it can only compute the normal, point, direction (geometry)
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let l = self.left - self.origin;
        let r = self.right - self.origin;
        let normal = r.cross(l);

        if normal.dot(ray.direction) == 0.0 {
            return None;
        }

        let t = normal.dot(self.origin - ray.origin) / normal.dot(ray.direction);
        let point = ray.origin + ray.direction * t;

        // Test inside (dot both sides of linear equation al + br = p with l and r to obtain 2 scalar equations with 2 unknowns; compute determinant; non-zero (within threshold) means inside bounds)

        // Geometry {
        //    point,
        //    direction: ray.direction * t,
        //    normal,
        // }

        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ShapeConfig {
    Sphere(SphereConfig),
    Parallelogram(ParallelogramConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SphereConfig {
    center: PointConfig,
    radius: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParallelogramConfig {
    origin: PointConfig,
    a: PointConfig,
    b: PointConfig,
}
