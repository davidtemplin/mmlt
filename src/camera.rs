use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Geometry,
    image::PixelCoordinates,
    interaction::{CameraInteraction, Interaction},
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    util,
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
        let u = self.u * (x - self.pixel_width / 2.0);
        let v = self.v * (y - self.pixel_height / 2.0);
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
        let fov = config.field_of_view.configure();
        let direction = Vector::configure(&config.direction);
        PinholeCamera::new(origin, direction, fov, image_width, image_height)
    }

    pub fn new(
        origin: Point,
        direction: Vector,
        field_of_view: f64,
        image_width: usize,
        image_height: usize,
    ) -> PinholeCamera {
        let pixel_width = image_width as f64;
        let pixel_height = image_height as f64;
        let distance = pixel_height / (2.0 * (field_of_view / 2.0).tan());
        let (u, v, w) = util::orthonormal_basis(direction);
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

impl FieldOfViewConfig {
    pub fn configure(&self) -> f64 {
        match self.unit {
            AngleUnitConfig::Degrees => self.value * (PI / 180.0),
            AngleUnitConfig::Radians => self.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PinholeCamera;
    use crate::{
        camera::{AngleUnitConfig, Camera, FieldOfViewConfig, PinholeCameraConfig},
        interaction::Interaction,
        sampler::test::MockSampler,
        spectrum::Spectrum,
        vector::{Point, PointConfig, Vector, VectorConfig},
    };
    use std::f64::consts::PI;

    #[test]
    fn test_pinhole_camera_configure() {
        let config = PinholeCameraConfig {
            origin: PointConfig {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            direction: VectorConfig {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            field_of_view: FieldOfViewConfig {
                value: 60.0,
                unit: AngleUnitConfig::Degrees,
            },
        };
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::configure(config, image_width, image_height);
        assert_eq!(camera.id, "camera");
        let origin = Vector::new(0.0, 0.0, 0.0);
        assert_eq!(camera.origin, origin);
        let h = image_height as f64;
        let w = image_width as f64;
        let field_of_view = 60.0 * PI / 180.0;
        let a = field_of_view / 2.0;
        let distance = h / (2.0 * a.tan());
        assert_eq!(camera.distance, distance);
        assert_eq!(camera.pixel_height, h);
        assert_eq!(camera.pixel_width, w);
        assert_eq!(camera.u, Vector::new(1.0, 0.0, 0.0));
        assert_eq!(camera.v, Vector::new(0.0, 1.0, 0.0));
        let direction = Vector::new(0.0, 0.0, 1.0);
        assert_eq!(camera.w, direction);
    }

    #[test]
    fn test_pinhole_camera_new() {
        let origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(0.0, 0.0, 1.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera =
            PinholeCamera::new(origin, direction, field_of_view, image_width, image_height);
        assert_eq!(camera.id, "camera");
        assert_eq!(camera.origin, origin);
        let h = image_height as f64;
        let w = image_width as f64;
        let a = field_of_view / 2.0;
        let distance = h / (2.0 * a.tan());
        assert_eq!(camera.distance, distance);
        assert_eq!(camera.pixel_height, h);
        assert_eq!(camera.pixel_width, w);
        assert_eq!(camera.u, Vector::new(1.0, 0.0, 0.0));
        assert_eq!(camera.v, Vector::new(0.0, 1.0, 0.0));
        assert_eq!(camera.w, direction);
    }

    #[test]
    fn test_pinhole_camera_importance() {
        let origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(0.0, 0.0, 1.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera =
            PinholeCamera::new(origin, direction, field_of_view, image_width, image_height);
        let d = Vector::new(1.0, 1.0, 1.0);
        let c = d.norm().dot(direction);
        let w = image_width as f64;
        let h = image_height as f64;
        let a = w * h;
        let i = 1.0 / (a * c * c * c * c);
        let importance = Spectrum::fill(i);
        assert_eq!(camera.importance(origin, d), importance);
    }

    #[test]
    fn test_pinhole_camera_probability() {
        let origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(0.0, 0.0, 1.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera =
            PinholeCamera::new(origin, direction, field_of_view, image_width, image_height);
        let r = Vector::new(1.0, 1.0, 1.0);
        let c = r.norm().dot(direction);
        let w = image_width as f64;
        let h = image_height as f64;
        let a = w * h;
        let half_fov = field_of_view / 2.0;
        let distance = h / (2.0 * half_fov.tan());
        let d = distance / c;
        let probability = Some((d * d) / (a * c));
        assert_eq!(camera.probability(origin, r), probability);
    }

    #[test]
    fn test_pinhole_camera_sample_interaction() {
        let origin = Point::new(0.0, 0.0, 0.0);
        let direction = Vector::new(0.0, 0.0, 1.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera =
            PinholeCamera::new(origin, direction, field_of_view, image_width, image_height);
        let mut sampler = MockSampler::new();
        sampler.add(0.5);
        sampler.add(0.5);
        let interaction = camera.sample_interaction(&mut sampler);
        match interaction {
            Interaction::Camera(camera_interaction) => {
                let h = image_height as f64;
                let half_fov = field_of_view / 2.0;
                let distance = h / (2.0 * half_fov.tan());
                assert_eq!(camera_interaction.pixel_coordinates.x, 256);
                assert_eq!(camera_interaction.pixel_coordinates.y, 256);
                assert_eq!(camera_interaction.geometry.normal, direction);
                assert_eq!(camera_interaction.geometry.point, distance * origin);
                assert_eq!(camera_interaction.geometry.direction, direction);
            }
            _ => panic!(),
        }
    }
}
