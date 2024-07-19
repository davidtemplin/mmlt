use std::cell::OnceCell;

use crate::{
    bsdf::{Bsdf, EvaluationContext},
    camera::Camera,
    geometry::Geometry,
    light::Light,
    object::Object,
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    types::PathType,
    vector::{Point2, Vector3},
};

#[derive(Debug)]
pub struct CameraInteraction<'a> {
    pub camera: &'a (dyn Camera + 'a),
    pub geometry: Geometry,
    pub pixel_coordinates: Point2,
}

#[derive(Debug)]
pub struct LightInteraction<'a> {
    pub light: &'a (dyn Light + 'a),
    pub geometry: Geometry,
}

#[derive(Debug)]
pub struct ObjectInteraction<'a> {
    pub object: &'a (dyn Object + 'a),
    pub geometry: Geometry,
    pub bsdf: OnceCell<Bsdf>,
}

#[derive(Debug)]
pub enum Interaction<'a> {
    Camera(CameraInteraction<'a>),
    Light(LightInteraction<'a>),
    Object(ObjectInteraction<'a>),
}

impl<'a> ObjectInteraction<'a> {
    pub fn get_bsdf(&self) -> &Bsdf {
        self.bsdf
            .get_or_init(|| self.object.compute_bsdf(self.geometry))
    }

    pub fn generate_ray(&self, path_type: PathType, sampler: &mut dyn Sampler) -> Option<Ray> {
        let wx = self.geometry.direction * -1.0;
        let direction = self
            .get_bsdf()
            .sample_direction(wx, path_type, sampler)?
            .norm();
        let ray = Ray {
            origin: self.geometry.point,
            direction,
        };
        Some(ray)
    }

    pub fn sampling_pdf(&self, wx: Vector3, path_type: PathType) -> Option<f64> {
        self.get_bsdf().sampling_pdf(wx, path_type)
    }

    pub fn pdf(&self, wo: Vector3, wi: Vector3, path_type: PathType) -> Option<f64> {
        self.get_bsdf().pdf(wo, wi, path_type)
    }

    pub fn reflectance(&self, wo: Vector3, wi: Vector3, context: EvaluationContext) -> Spectrum {
        self.get_bsdf().evaluate(wo, wi, context)
    }
}

impl<'a> Interaction<'a> {
    pub fn initial_ray(&self) -> Option<Ray> {
        match self {
            Interaction::Camera(i) => {
                let ray = Ray::new(i.geometry.point, i.geometry.direction);
                Some(ray)
            }
            Interaction::Light(i) => {
                let ray = Ray::new(i.geometry.point, i.geometry.direction);
                Some(ray)
            }
            _ => None,
        }
    }

    pub fn generate_ray(&self, path_type: PathType, sampler: &mut dyn Sampler) -> Option<Ray> {
        match self {
            Interaction::Camera(_) => None,
            Interaction::Light(_) => None,
            Interaction::Object(object_interaction) => {
                object_interaction.generate_ray(path_type, sampler)
            }
        }
    }

    pub fn id(&self) -> &String {
        match self {
            Interaction::Camera(i) => i.camera.id(),
            Interaction::Light(i) => i.light.id(),
            Interaction::Object(i) => i.object.id(),
        }
    }

    pub fn geometry(&self) -> Geometry {
        match self {
            Interaction::Camera(i) => i.geometry,
            Interaction::Light(i) => i.geometry,
            Interaction::Object(i) => i.geometry,
        }
    }

    pub fn distance(&self) -> f64 {
        match self {
            Interaction::Camera(i) => i.geometry.direction.len(),
            Interaction::Light(i) => i.geometry.direction.len(),
            Interaction::Object(i) => i.geometry.direction.len(),
        }
    }

    pub fn is_camera(&self) -> bool {
        match self {
            Interaction::Camera(_) => true,
            _ => false,
        }
    }

    pub fn is_light(&self) -> bool {
        match self {
            Interaction::Light(_) => true,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Interaction::Object(_) => true,
            _ => false,
        }
    }

    pub fn set_direction(&mut self, direction: Vector3) {
        self.geometry().set_direction(direction);
    }
}
