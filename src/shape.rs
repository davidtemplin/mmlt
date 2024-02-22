use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    ray::Ray,
    sampler::Sampler,
    util,
    vector::{Point, PointConfig, Vector},
};

pub trait Shape {
    fn area(&self) -> f64;
    fn sample_intersection(&self, sampler: &mut dyn Sampler) -> Geometry;
    fn intersect(&self, ray: Ray) -> Option<Geometry>;
}

pub struct Sphere {
    center: Point,
    radius: f64,
}

impl Sphere {
    pub fn configure(config: &SphereConfig) -> Sphere {
        Sphere {
            center: Point::configure(&config.center),
            radius: config.radius,
        }
    }
}

impl Shape for Sphere {
    fn area(&self) -> f64 {
        4.0 * PI * self.radius
    }

    fn sample_intersection(&self, sampler: &mut dyn Sampler) -> Geometry {
        let u1 = sampler.sample(0.0..1.0);
        let u2 = sampler.sample(0.0..1.0);
        let point = self.center + util::uniform_sample_sphere(u1, u2) * self.radius;
        Geometry {
            point,
            direction: point.norm(),
            normal: point.norm(),
        }
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

impl Parallelogram {
    pub fn configure(config: &ParallelogramConfig) -> Parallelogram {
        Parallelogram {
            origin: Point::configure(&config.origin),
            a: Vector::configure(&config.a),
            b: Vector::configure(&config.b),
        }
    }
}

impl Shape for Parallelogram {
    fn area(&self) -> f64 {
        let left_length = self.a.len();
        let right_length = self.b.len();
        let area = left_length * right_length;
        area
    }

    fn sample_intersection(&self, sampler: &mut dyn Sampler) -> Geometry {
        let a = sampler.sample(0.0..1.0);
        let b = sampler.sample(0.0..1.0);
        let point = self.a * a + self.b * b;
        let normal = self.a.cross(self.b);
        Geometry {
            point,
            normal,
            direction: normal,
        }
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

impl ShapeConfig {
    pub fn configure(&self) -> Box<dyn Shape> {
        match self {
            ShapeConfig::Parallelogram(c) => Box::new(Parallelogram::configure(c)),
            ShapeConfig::Sphere(c) => Box::new(Sphere::configure(c)),
        }
    }
}
