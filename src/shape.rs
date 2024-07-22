use std::{f64::consts::PI, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    ray::Ray,
    sampler::Sampler,
    util,
    vector::{Point3, Point3Config},
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ShapeConfig {
    Sphere(SphereConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SphereConfig {
    center: Point3Config,
    radius: f64,
}

impl ShapeConfig {
    pub fn configure(&self) -> Box<dyn Shape> {
        match self {
            ShapeConfig::Sphere(c) => Box::new(Sphere::configure(c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{Shape, Sphere};
    use crate::{
        approx::ApproxEq,
        geometry::Geometry,
        ray::Ray,
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
}
