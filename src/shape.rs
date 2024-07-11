use std::{f64::consts::PI, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    ray::Ray,
    sampler::Sampler,
    util,
    vector::{Point3, Point3Config, Vector3},
};

pub trait Shape: fmt::Debug {
    fn area(&self) -> f64;
    fn sample_geometry(&self, sampler: &mut dyn Sampler) -> Geometry;
    fn intersect(&self, ray: Ray) -> Option<Geometry>;
}

#[derive(Debug)]
pub struct Sphere {
    center: Point3,
    radius: f64,
}

impl Sphere {
    pub fn configure(config: &SphereConfig) -> Sphere {
        Sphere::new(Point3::configure(&config.center), config.radius)
    }

    pub fn new(center: Point3, radius: f64) -> Sphere {
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

#[derive(Debug)]
pub struct Parallelogram {
    origin: Point3,
    a: Vector3,
    b: Vector3,
}

impl Parallelogram {
    pub fn configure(config: &ParallelogramConfig) -> Parallelogram {
        Parallelogram::new(
            Point3::configure(&config.origin),
            Vector3::configure(&config.a),
            Vector3::configure(&config.b),
        )
    }

    pub fn new(origin: Point3, a: Vector3, b: Vector3) -> Parallelogram {
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
        let point = self.origin + self.a * a + self.b * b;
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
        let threshold = 1e-6;

        if nd.abs() < threshold {
            return None;
        }

        let t = normal.dot(self.origin - ray.origin) / nd;

        if t <= 0.0 {
            return None;
        }

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
    center: Point3Config,
    radius: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParallelogramConfig {
    origin: Point3Config,
    a: Point3Config,
    b: Point3Config,
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
        sampler::test::MockSampler,
        vector::{Point3, Vector3},
    };

    #[test]
    fn test_sphere_area() {
        let center = Point3::new(10.0, 10.0, 10.0);
        let radius = 2.0;
        let sphere = Sphere::new(center, radius);
        let area = sphere.area();
        assert_eq!(area, 16.0 * PI);
    }

    #[test]
    fn test_sphere_insersect() {
        let tolerance = 1e-8;

        let center = Point3::new(10.0, 0.0, 0.0);
        let radius = 1.0;
        let sphere = Sphere::new(center, radius);
        let origin = Point3::new(0.0, 0.0, 0.0);
        let direction = Vector3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, direction);
        let actual = sphere.intersect(ray).unwrap();
        let expected = Geometry {
            point: Point3::new(9.0, 0.0, 0.0),
            normal: Vector3::new(-1.0, 0.0, 0.0),
            direction: Vector3::new(9.0, 0.0, 0.0),
        };
        assert!(actual.approx_eq(expected, tolerance));

        let center = Point3::new(10.0, 10.0, 10.0);
        let sphere = Sphere::new(center, radius);
        let direction = Vector3::new(1.0, 1.0, 1.0).norm();
        let ray = Ray::new(origin, direction);
        let actual = sphere.intersect(ray).unwrap();
        let offset = Vector3::new(-1.0, -1.0, -1.0).norm();
        let expected = Geometry {
            point: center + offset,
            normal: offset,
            direction: center + offset,
        };
        assert!(actual.approx_eq(expected, tolerance));

        let center = Point3::new(10.0, 10.0, 10.0);
        let radius = 2.0;
        let sphere = Sphere::new(center, radius);
        let origin = Point3::new(1.0, 2.0, -3.0);
        let offset = Vector3::new(-1.0, -1.0, 1.0).norm() * radius;
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
        let origin = Point3::new(10.0, -1.0, 0.0);
        let a = Vector3::new(0.0, 0.0, 2.0);
        let b = Vector3::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        assert_eq!(parallelogram.area(), a.len() * b.len());
    }

    #[test]
    fn test_parallelogram_intersect() {
        let tolerance = 1e-8;
        let origin = Point3::new(10.0, -1.0, 0.0);
        let a = Vector3::new(0.0, 0.0, 2.0);
        let b = Vector3::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        let ray_origin = Point3::new(0.0, 0.0, 0.0);
        let direction = Vector3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(ray_origin, direction);
        let actual = parallelogram.intersect(ray).unwrap();
        let expected = Geometry {
            point: Point3::new(10.0, 0.0, 0.0),
            normal: Vector3::new(-1.0, 0.0, 0.0),
            direction: Vector3::new(10.0, 0.0, 0.0),
        };
        assert!(actual.approx_eq(expected, tolerance));

        let origin = Point3::new(10.0, 10.0, 10.0);
        let a = Vector3::new(0.0, 0.0, 2.0);
        let b = Vector3::new(0.0, 2.0, 0.0);
        let parallelogram = Parallelogram::new(origin, a, b);
        let ray_origin = Point3::new(1.0, 2.0, 3.0);
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

    #[test]
    fn test_parallelogram_sample_geometry() {
        let origin = Point3::new(0.3, 1.0, 0.3);
        let a = Vector3::new(0.4, 0.0, 0.0);
        let b = Vector3::new(0.0, 0.0, 0.4);
        let parallelogram = Parallelogram::new(origin, a, b);
        let mut sampler = MockSampler::new();
        sampler.add(0.6);
        sampler.add(0.25);
        let geometry = parallelogram.sample_geometry(&mut sampler);
        assert_eq!(geometry.normal, Vector3::new(0.0, -1.0, 0.0));
        assert_eq!(geometry.direction, Vector3::new(0.0, -1.0, 0.0));
        assert!(a.norm().dot(geometry.point - origin) < 1.0);
        assert!(b.norm().dot(geometry.point - origin) < 1.0);
    }
}
