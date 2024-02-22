use std::cell::OnceCell;

use crate::{
    bsdf::Bsdf,
    camera::Camera,
    geometry::Geometry,
    light::Light,
    object::Object,
    ray::Ray,
    sampler::Sampler,
    spectrum::Spectrum,
    vector::{Point, Vector},
};

pub enum Orientation {
    Camera,
    Light,
}

pub struct CameraInteraction<'a> {
    pub camera: &'a (dyn Camera + 'a),
    pub geometry: Geometry,
}

pub struct LightInteraction<'a> {
    pub light: &'a (dyn Light + 'a),
    pub geometry: Geometry,
}

pub struct ObjectInteraction<'a> {
    pub object: &'a (dyn Object + 'a),
    pub geometry: Geometry,
    pub bsdf: OnceCell<Bsdf>,
}

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

    pub fn generate_ray(&self, sampler: &dyn Sampler) -> Ray {
        self.get_bsdf().generate_ray(sampler)
    }

    pub fn probability(&self, wo: Vector, wi: Vector) -> f64 {
        self.get_bsdf().probability(wo, wi)
    }

    pub fn reflectance(&self, wo: Vector, wi: Vector) -> Spectrum {
        self.get_bsdf().evaluate(wo, wi)
    }
}

impl<'a> Interaction<'a> {
    pub fn generate_ray(&self, sampler: &dyn Sampler) -> Option<Ray> {
        match self {
            Interaction::Camera(_) => None,
            Interaction::Light(_) => None,
            Interaction::Object(object_interaction) => {
                Some(object_interaction.generate_ray(sampler))
            }
        }
    }

    pub fn id(&self) -> u64 {
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
}
