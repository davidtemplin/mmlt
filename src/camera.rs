use crate::{
    intersection::{CameraIntersection, Intersection, Orientation},
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    vector::{Point, Vector},
};

pub trait Camera {
    fn importance(&self, point: Point, direction: Vector) -> Spectrum;
    fn probability(&self, point: Point, direction: Vector) -> f64;
    fn sample_intersection(&self, sampler: &mut dyn Sampler) -> Intersection;
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
    fn id(&self) -> u64;
}

pub struct PinholeCamera {
    id: u64,
    u: Vector,
    v: Vector,
    w: Vector,
    origin: Point,
    distance: f64,
    pixel_width: f64,
    pixel_height: f64,
}

impl Camera for PinholeCamera {
    fn importance(&self, _point: Point, direction: Vector) -> Spectrum {
        let c = direction.norm().dot(self.w);
        let a = self.pixel_width * self.pixel_height;
        let c2 = c * c;
        let c4 = c2 * c2;
        Spectrum::fill(1.0 / (a * c4))
    }

    fn probability(&self, _point: Point, direction: Vector) -> f64 {
        let c = direction.norm().dot(self.w);
        let d = self.distance / c;
        let d2 = d * d;
        let a = self.pixel_width * self.pixel_height;
        d2 / (a * c)
    }

    fn sample_intersection(&self, sampler: &mut dyn Sampler) -> Intersection {
        let width = self.pixel_width / 2.0;
        let height = self.pixel_height / 2.0;
        let u = self.u * sampler.sample(-width..width);
        let v = self.v * sampler.sample(-height..height);
        let w = self.w * self.distance;
        let direction = (u + v + w).norm();
        let camera_intersection = CameraIntersection {
            camera: self,
            point: self.origin,
            direction,
            normal: self.w,
            orientation: Orientation::Camera,
        };
        Intersection::Camera(camera_intersection)
    }

    fn intersect(&self, _ray: Ray) -> Option<Intersection> {
        None
    }

    fn id(&self) -> u64 {
        self.id
    }
}
