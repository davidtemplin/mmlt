use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    image::PixelCoordinates,
    interaction::{CameraInteraction, Interaction},
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    vector::{Point, PointConfig, Vector, VectorConfig},
};

pub trait Camera {
    fn importance(&self, point: Point, direction: Vector) -> Spectrum;
    fn probability(&self, point: Point, direction: Vector) -> Option<f64>;
    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction;
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn id(&self) -> &String;
}

pub struct PinholeCamera {
    id: String,
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

    fn probability(&self, _point: Point, direction: Vector) -> Option<f64> {
        let c = direction.norm().dot(self.w);
        let d = self.distance / c;
        let d2 = d * d;
        let a = self.pixel_width * self.pixel_height;
        let p = d2 / (a * c);
        Some(p)
    }

    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction {
        let x = sampler.sample(0.0..self.pixel_width);
        let y = sampler.sample(0.0..self.pixel_height);
        let u = self.u * (x - self.pixel_width);
        let v = self.v * (y - self.pixel_height);
        let w = self.w * self.distance;
        let direction = (u + v + w).norm();
        let pixel_coordinates = PixelCoordinates::new(x as usize, y as usize);
        let camera_interaction = CameraInteraction {
            camera: self,
            geometry: Geometry {
                point: self.origin,
                direction,
                normal: self.w,
            },
            pixel_coordinates,
        };
        Interaction::Camera(camera_interaction)
    }

    fn intersect(&self, _ray: Ray) -> Option<Interaction> {
        None
    }

    fn id(&self) -> &String {
        &self.id
    }
}

impl PinholeCamera {
    pub fn configure(
        config: PinholeCameraConfig,
        image_width: usize,
        image_height: usize,
    ) -> PinholeCamera {
        let origin = Vector::configure(&config.origin);
        let pixel_width = image_width as f64;
        let pixel_height = image_height as f64;
        let fov = match config.field_of_view.unit {
            AngleUnitConfig::Degrees => config.field_of_view.value * (PI / 180.0),
            AngleUnitConfig::Radians => config.field_of_view.value,
        };
        let distance = pixel_height / (2.0 * (fov / 2.0).tan());
        let w = Vector::configure(&config.direction).norm();
        let u = w.cross(Vector::new(0.0, 1.0, 0.0)).norm();
        let v = u.cross(w);
        PinholeCamera {
            id: String::from("camera"),
            u,
            v,
            w,
            origin,
            distance,
            pixel_width,
            pixel_height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum CameraConfig {
    Pinhole(PinholeCameraConfig),
}

impl CameraConfig {
    pub fn configure(self, image_width: usize, image_height: usize) -> impl Camera {
        match self {
            CameraConfig::Pinhole(config) => {
                PinholeCamera::configure(config, image_width, image_height)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PinholeCameraConfig {
    origin: PointConfig,
    direction: VectorConfig,
    field_of_view: FieldOfViewConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AngleUnitConfig {
    Degrees,
    Radians,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldOfViewConfig {
    value: f64,
    unit: AngleUnitConfig,
}
