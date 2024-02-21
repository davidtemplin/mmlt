use crate::{
    camera::Camera,
    light::Light,
    object::Object,
    ray::Ray,
    sampler::Sampler,
    vector::{Point, Vector},
};

pub enum Orientation {
    Camera,
    Light,
}

pub struct CameraInteraction<'a> {
    pub camera: &'a (dyn Camera + 'a),
    pub point: Point,
    pub direction: Vector,
    pub normal: Vector,
    pub orientation: Orientation,
}

pub struct LightInteraction<'a> {
    pub light: &'a (dyn Light + 'a),
    pub point: Point,
    pub direction: Vector,
    pub normal: Vector,
    pub orientation: Orientation,
}

pub struct ObjectInteraction<'a> {
    pub object: &'a (dyn Object + 'a),
    pub point: Point,
    pub normal: Vector,
    pub direction: Vector,
    pub orientation: Orientation,
}

pub enum Interaction<'a> {
    Camera(CameraInteraction<'a>),
    Light(LightInteraction<'a>),
    Object(ObjectInteraction<'a>),
}

impl<'a> Interaction<'a> {
    pub fn generate_ray(&self, sampler: &dyn Sampler) -> Option<Ray> {
        match self {
            Interaction::Camera(_) => None,
            Interaction::Light(_) => None,
            Interaction::Object(object_interaction) => {
                Some(object_interaction.object.generate_ray(
                    object_interaction.normal,
                    object_interaction.direction,
                    sampler,
                ))
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

    pub fn normal(&self) -> Vector {
        match self {
            Interaction::Camera(c) => c.normal,
            Interaction::Light(l) => l.normal,
            Interaction::Object(o) => o.normal,
        }
    }

    pub fn point(&self) -> Point {
        match self {
            Interaction::Camera(c) => c.point,
            Interaction::Light(l) => l.point,
            Interaction::Object(o) => o.point,
        }
    }

    pub fn distance(&self) -> f64 {
        match self {
            Interaction::Camera(i) => i.direction.len(),
            Interaction::Light(i) => i.direction.len(),
            Interaction::Object(i) => i.direction.len(),
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
