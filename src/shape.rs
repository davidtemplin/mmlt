use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::{self, Geometry},
    interaction::{Interaction, ObjectInteraction, Orientation},
    ray::Ray,
    sampler::Sampler,
    vector::{Point, PointConfig, Vector},
};

pub trait Shape {
    fn area(&self) -> f64;
    fn sample_intersection(&self, sampler: &dyn Sampler) -> Geometry;
    fn intersect(&self, ray: Ray) -> Option<Geometry>;
}

pub struct Sphere {
    center: Point,
    radius: f64,
}

impl Shape for Sphere {
    fn area(&self) -> f64 {
        4.0 * PI * self.radius
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Geometry {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Geometry> {
        let c = self.center - ray.origin;
        let b = c.dot(ray.direction);
        let mut det = b * b - c.dot(c) + self.radius * self.radius;
        if det < 0.0 {
            return None;
        }
        det = det.sqrt();
        let threshold = 1e-4;
        let mut t = b - det;
        if t <= threshold {
            t = b + det;
            if t <= threshold {
                return None;
            }
        }

        let point = ray.origin + ray.direction * t;
        let normal = (point - self.center).norm();
        let direction = ray.direction * t;

        let geometry = Geometry {
            point,
            normal,
            direction,
        };

        Some(geometry)
    }
}

pub struct Parallelogram {
    origin: Point,
    a: Vector,
    b: Vector,
}

impl Shape for Parallelogram {
    fn area(&self) -> f64 {
        let left_length = self.a.len();
        let right_length = self.b.len();
        let area = left_length * right_length;
        area
    }

    fn sample_intersection(&self, sampler: &dyn Sampler) -> Geometry {
        todo!()
    }

    fn intersect(&self, ray: Ray) -> Option<Geometry> {
        let normal = self.a.cross(self.b);

        let nd = normal.dot(ray.direction);

        if nd == 0.0 {
            return None;
        }

        let t = normal.dot(self.origin - ray.origin) / nd;

        let point = ray.origin + ray.direction * t;

        let p = point - self.origin;

        let aa = self.a.dot(self.a);
        let ba = self.b.dot(self.a);
        let ab = self.a.dot(self.b);
        let bb = self.b.dot(self.b);
        let pa = p.dot(self.a);
        let pb = p.dot(self.b);

        let da = pa * bb - ba * pb;
        let db = aa * pb - pa * ab;
        let d = aa * bb - ba * ab;

        let sa = da / d;
        let sb = db / d;

        let threshold = 1e-4;
        let min = -threshold;
        let max = 1.0 + threshold;
        let range = min..max;

        if !range.contains(&sa) || !range.contains(&sb) {
            return None;
        }

        let geometry = Geometry {
            point,
            direction: ray.direction * t,
            normal,
        };

        Some(geometry)
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
