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
    fn sample_geometry(&self, sampler: &mut dyn Sampler) -> Geometry;
    fn intersect(&self, ray: Ray) -> Option<Geometry>;
}

pub struct Sphere {
    center: Point,
    radius: f64,
}

impl Sphere {
    pub fn configure(config: &SphereConfig) -> Sphere {
        Sphere::new(Point::configure(&config.center), config.radius)
    }

    pub fn new(center: Point, radius: f64) -> Sphere {
        Sphere { center, radius }
    }
}

impl Shape for Sphere {
    fn area(&self) -> f64 {
        4.0 * PI * self.radius * self.radius
    }

    fn sample_geometry(&self, sampler: &mut dyn Sampler) -> Geometry {
        let direction = util::uniform_sample_sphere(sampler) * self.radius;
        let point = self.center + direction;
        Geometry {
            point,
            direction,
            normal: direction.norm(),
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
        Parallelogram::new(
            Point::configure(&config.origin),
            Vector::configure(&config.a),
            Vector::configure(&config.b),
        )
    }

    pub fn new(origin: Point, a: Vector, b: Vector) -> Parallelogram {
        Parallelogram { origin, a, b }
    }
}

impl Shape for Parallelogram {
    fn area(&self) -> f64 {
        let left_length = self.a.len();
        let right_length = self.b.len();
        let area = left_length * right_length;
        area
    }

    fn sample_geometry(&self, sampler: &mut dyn Sampler) -> Geometry {
        let a = sampler.sample(0.0..1.0);
        let b = sampler.sample(0.0..1.0);
        let point = self.a * a + self.b * b;
        let normal = self.a.cross(self.b).norm();
        Geometry {
            point,
            normal,
            direction: normal,
        }
    }

    fn intersect(&self, ray: Ray) -> Option<Geometry> {
        let normal = self.a.cross(self.b).norm();

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

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{Parallelogram, Shape, Sphere};
    use crate::{
        approx::ApproxEq,
        geometry::Geometry,
        ray::Ray,
        vector::{Point, Vector},
    };

    #[test]
    fn test_sphere_area() {
        let center = Point::new(10.0, 10.0, 10.0);
        let radius = 2.0;
        let sphere = Sphere::new(center, radius);
        let area = sphere.area();
        assert_eq!(area, 16.0 * PI);
    }

    #[test]
    fn test_sphere_insersect() {
        let tolerance = 1e-8;

        let center = Point::new(10.0, 0.0, 0.0);
        let radius = 1.0;
        let sphere = Sphere::new(center, radius);
        let origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, direction);
        let actual = sphere.intersect(ray).unwrap();
        let expected = Geometry {
            point: Point::new(9.0, 0.0, 0.0),
            normal: Vector::new(-1.0, 0.0, 0.0),
            direction: Vector::new(9.0, 0.0, 0.0),
        };
        assert!(actual.approx_eq(expected, tolerance));

        let center = Point::new(10.0, 10.0, 10.0);
        let sphere = Sphere::new(center, radius);
        let direction = Vector::new(1.0, 1.0, 1.0).norm();
        let ray = Ray::new(origin, direction);
        let actual = sphere.intersect(ray).unwrap();
        let offset = Vector::new(-1.0, -1.0, -1.0).norm();
        let expected = Geometry {
            point: center + offset,
            normal: offset,
            direction: center + offset,
        };
        assert!(actual.approx_eq(expected, tolerance));

        let center = Point::new(10.0, 10.0, 10.0);
        let radius = 2.0;
        let sphere = Sphere::new(center, radius);
        let origin = Point::new(1.0, 2.0, -3.0);
        let offset = Vector::new(-1.0, -1.0, 1.0).norm() * radius;
        let direction = (center + offset - origin).norm();
        let ray = Ray::new(origin, direction);
        let actual = sphere.intersect(ray).unwrap();
        let expected = Geometry {
            point: center + offset,
            normal: offset.norm(),
            direction: center + offset - origin,
        };
        assert!(actual.approx_eq(expected, tolerance));
    }

    #[test]
    fn test_parallelogram_area() {
        let origin = Point::new(10.0, -1.0, 0.0);
        let a = Vector::new(0.0, 0.0, 2.0);
        let b = Vector::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        assert_eq!(parallelogram.area(), a.len() * b.len());
    }

    #[test]
    fn test_parallelogram_intersect() {
        let tolerance = 1e-8;
        let origin = Point::new(10.0, -1.0, 0.0);
        let a = Vector::new(0.0, 0.0, 2.0);
        let b = Vector::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        let ray_origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(ray_origin, direction);
        let actual = parallelogram.intersect(ray).unwrap();
        let expected = Geometry {
            point: Point::new(10.0, 0.0, 0.0),
            normal: Vector::new(-1.0, 0.0, 0.0),
            direction: Vector::new(10.0, 0.0, 0.0),
        };
        assert!(actual.approx_eq(expected, tolerance));

        let origin = Point::new(10.0, 10.0, 10.0);
        let a = Vector::new(0.0, 0.0, 2.0);
        let b = Vector::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        let ray_origin = Point::new(1.0, 2.0, 3.0);
        let target = origin + 0.5 * a + 0.5 * b;
        let direction = (target - ray_origin).norm();
        let ray = Ray::new(ray_origin, direction);
        let actual = parallelogram.intersect(ray).unwrap();
        let expected = Geometry {
            point: target,
            normal: a.cross(b).norm(),
            direction: target - ray_origin,
        };
        assert!(actual.approx_eq(expected, tolerance));
    }
}
