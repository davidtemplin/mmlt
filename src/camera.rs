use std::{f64::consts::PI, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    approx::ApproxEq,
    geometry::Geometry,
    interaction::{CameraInteraction, Interaction},
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    util,
    vector::{Point2, Point3, Point3Config, Vector3},
};

pub trait Camera: fmt::Debug {
    fn importance(&self, point: Point3, direction: Vector3) -> Spectrum;
    fn positional_pdf(&self, point: Point3) -> Option<f64>;
    fn directional_pdf(&self, direction: Vector3) -> Option<f64>;
    fn sample_interaction(&self, sampler: &mut dyn Sampler) -> Interaction;
    fn intersect(&self, ray: Ray) -> Option<Interaction>;
    fn id(&self) -> &String;
}

#[derive(Debug)]
pub struct PinholeCamera {
    id: String,
    u: Vector3,
    v: Vector3,
    w: Vector3,
    origin: Point3,
    distance: f64,
    pixel_width: f64,
    pixel_height: f64,
}

impl Camera for PinholeCamera {
    fn importance(&self, _point: Point3, direction: Vector3) -> Spectrum {
        let c = direction.norm().dot(self.w);
        let a = self.pixel_width * self.pixel_height;
        let c4 = c * c * c * c;
        let d2 = self.distance * self.distance;
        Spectrum::fill(d2 / (a * c4))
    }

    fn positional_pdf(&self, _: Point3) -> Option<f64> {
        Some(1.0)
    }

    fn directional_pdf(&self, direction: Vector3) -> Option<f64> {
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
        let v = -self.v * (y - self.pixel_height / 2.0);
        let w = self.w * self.distance;
        let direction = (u + v + w).norm();
        let pixel_coordinates = Point2::new(x, y);
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

    fn intersect(&self, ray: Ray) -> Option<Interaction> {
        let o = self.origin - ray.origin;
        let t = if ray.direction.x != 0.0 && o.x != 0.0 {
            o.x / ray.direction.x
        } else if ray.direction.y != 0.0 && o.y != 0.0 {
            o.y / ray.direction.y
        } else if ray.direction.z != 0.0 && o.z != 0.0 {
            o.z / ray.direction.z
        } else {
            0.0
        };
        let i = ray.origin + t * ray.direction;
        let tolerance = 1e-6;
        if !i.approx_eq(self.origin, tolerance) {
            return None;
        }
        let d = (ray.origin - self.origin).norm();
        let screen_center = self.w * self.distance;
        let wd = self.w.dot(d);
        if wd == 0.0 {
            return None;
        }
        let t = self.w.dot(screen_center) / wd;
        if t <= 0.0 {
            return None;
        }
        let p = t * d - screen_center;
        let px = self.u.dot(p) + self.pixel_width * 0.5;
        let py = -self.v.dot(p) + self.pixel_height * 0.5;
        if (0.0..self.pixel_width).contains(&px) && (0.0..self.pixel_height).contains(&py) {
            let camera_interaction = CameraInteraction {
                camera: self,
                geometry: Geometry {
                    point: self.origin,
                    direction: ray.origin - self.origin,
                    normal: self.w,
                },
                pixel_coordinates: Point2::new(px, py),
            };
            let interaction = Interaction::Camera(camera_interaction);
            Some(interaction)
        } else {
            None
        }
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
        let origin = Vector3::configure(&config.origin);
        let fov = config.field_of_view.configure();
        let look_at = Vector3::configure(&config.look_at);
        PinholeCamera::new(origin, look_at, fov, image_width, image_height)
    }

    pub fn new(
        origin: Point3,
        look_at: Point3,
        field_of_view: f64,
        image_width: usize,
        image_height: usize,
    ) -> PinholeCamera {
        let pixel_width = image_width as f64;
        let pixel_height = image_height as f64;
        let distance = pixel_height / (2.0 * (field_of_view / 2.0).tan());
        let direction = look_at - origin;
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
    origin: Point3Config,
    look_at: Point3Config,
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
        ray::Ray,
        sampler::test::MockSampler,
        spectrum::Spectrum,
        vector::{Point3, Point3Config, Vector3},
    };
    use std::f64::consts::PI;

    #[test]
    fn test_pinhole_camera_configure() {
        let config = PinholeCameraConfig {
            origin: Point3Config {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            look_at: Point3Config {
                x: 0.0,
                y: 0.0,
                z: 50.0,
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
        let origin = Vector3::new(0.0, 0.0, 0.0);
        assert_eq!(camera.origin, origin);
        let h = image_height as f64;
        let w = image_width as f64;
        let field_of_view = 60.0 * PI / 180.0;
        let a = field_of_view / 2.0;
        let distance = h / (2.0 * a.tan());
        assert_eq!(camera.distance, distance);
        assert_eq!(camera.pixel_height, h);
        assert_eq!(camera.pixel_width, w);
        assert_eq!(camera.u, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(camera.v, Vector3::new(0.0, 1.0, 0.0));
        let direction = Vector3::new(0.0, 0.0, 1.0);
        assert_eq!(camera.w, direction);
    }

    #[test]
    fn test_pinhole_camera_new() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let look_at = Vector3::new(0.0, 0.0, 50.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        assert_eq!(camera.id, "camera");
        assert_eq!(camera.origin, origin);
        let h = image_height as f64;
        let w = image_width as f64;
        let a = field_of_view / 2.0;
        let distance = h / (2.0 * a.tan());
        assert_eq!(camera.distance, distance);
        assert_eq!(camera.pixel_height, h);
        assert_eq!(camera.pixel_width, w);
        assert_eq!(camera.u, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(camera.v, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(camera.w, Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_pinhole_camera_importance() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let look_at = Vector3::new(0.0, 0.0, 50.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        let d = Vector3::new(0.0, 0.25, 1.0);
        let direction = (look_at - origin).norm();
        let c = d.norm().dot(direction);
        let w = image_width as f64;
        let h = image_height as f64;
        let a = w * h;
        let half_fov = field_of_view / 2.0;
        let distance = h / (2.0 * half_fov.tan());
        let i = (distance * distance) / (a * c * c * c * c);
        let importance = Spectrum::fill(i);
        assert_eq!(camera.importance(origin, d), importance);
    }

    #[test]
    fn test_pinhole_camera_pdf() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let look_at = Vector3::new(0.0, 0.0, 50.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        let r = Vector3::new(0.0, 0.25, 1.0);
        let direction = (look_at - origin).norm();
        let c = r.norm().dot(direction);
        let w = image_width as f64;
        let h = image_height as f64;
        let a = w * h;
        let half_fov = field_of_view / 2.0;
        let distance = h / (2.0 * half_fov.tan());
        let d = distance / c;
        let pdf = Some((d * d) / (a * c));
        assert_eq!(camera.directional_pdf(r), pdf);
        assert_eq!(camera.positional_pdf(origin), Some(1.0));
    }

    #[test]
    fn test_pinhole_camera_sample_interaction() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let look_at = Vector3::new(0.0, 0.0, 50.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        let mut sampler = MockSampler::new();
        sampler.add(0.5);
        sampler.add(0.5);
        let interaction = camera.sample_interaction(&mut sampler);
        let direction = (look_at - origin).norm();
        match interaction {
            Interaction::Camera(camera_interaction) => {
                let h = image_height as f64;
                let half_fov = field_of_view / 2.0;
                let distance = h / (2.0 * half_fov.tan());
                assert_eq!(camera_interaction.pixel_coordinates.x, 256.0);
                assert_eq!(camera_interaction.pixel_coordinates.y, 256.0);
                assert_eq!(camera_interaction.geometry.normal, direction);
                assert_eq!(camera_interaction.geometry.point, distance * origin);
                assert_eq!(camera_interaction.geometry.direction, direction);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_pinhole_camera_intersect_hit() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let look_at = Vector3::new(0.0, 0.0, 50.0);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        let ray_origin = Point3::new(0.0, 0.0, 10.0);
        let ray_direction = Vector3::new(0.0, 0.0, -10.0).norm();
        let ray = Ray::new(ray_origin, ray_direction);
        let interaction = camera.intersect(ray);
        let direction = (look_at - origin).norm();
        match interaction {
            Some(Interaction::Camera(camera_interaction)) => {
                assert_eq!(camera_interaction.pixel_coordinates.x, 256.0);
                assert_eq!(camera_interaction.pixel_coordinates.y, 256.0);
                assert_eq!(camera_interaction.geometry.normal, direction);
                assert_eq!(
                    camera_interaction.geometry.point,
                    camera.distance * camera.origin
                );
                assert_eq!(camera_interaction.geometry.direction, ray_origin - origin);
            }
            _ => panic!("expected camera interaction"),
        }
    }

    #[test]
    fn test_pinhole_camera_intersect_miss() {
        let origin = Point3::new(0.5, 0.1, 0.01);
        let look_at = Vector3::new(0.5, 0.9, 0.5);
        let field_of_view = 60.0 * PI / 180.0;
        let image_width = 512;
        let image_height = 512;
        let camera = PinholeCamera::new(origin, look_at, field_of_view, image_width, image_height);
        let ray_origin = Point3::new(0.49277762278284754, 0.040182486681127116, 0.0);
        let ray_direction = (origin - ray_origin).norm();
        let ray = Ray::new(ray_origin, ray_direction);
        let interaction = camera.intersect(ray);
        assert!(interaction.is_none());
    }
}
