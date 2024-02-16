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

pub struct CameraIntersection<'a> {
    pub camera: &'a (dyn Camera + 'a),
    pub point: Point,
    pub direction: Vector,
    pub normal: Vector,
    pub orientation: Orientation,
}

pub struct LightIntersection<'a> {
    pub light: &'a (dyn Light + 'a),
    pub point: Point,
    pub direction: Vector,
    pub normal: Vector,
    pub orientation: Orientation,
}

pub struct ObjectIntersection<'a> {
    pub object: &'a (dyn Object + 'a),
    pub point: Point,
    pub normal: Vector,
    pub direction: Vector,
    pub orientation: Orientation,
}

pub enum Intersection<'a> {
    Camera(CameraIntersection<'a>),
    Light(LightIntersection<'a>),
    Object(ObjectIntersection<'a>),
}

impl<'a> Intersection<'a> {
    pub fn generate_ray(&self, sampler: &dyn Sampler) -> Option<Ray> {
        match self {
            Intersection::Camera(_) => None,
            Intersection::Light(_) => None,
            Intersection::Object(object_intersection) => {
                Some(object_intersection.object.generate_ray(
                    object_intersection.normal,
                    object_intersection.direction,
                    sampler,
                ))
            }
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Intersection::Camera(i) => i.camera.id(),
            Intersection::Light(i) => i.light.id(),
            Intersection::Object(i) => i.object.id(),
        }
    }

    pub fn normal(&self) -> Vector {
        match self {
            Intersection::Camera(c) => c.normal,
            Intersection::Light(l) => l.normal,
            Intersection::Object(o) => o.normal,
        }
    }

    pub fn point(&self) -> Point {
        match self {
            Intersection::Camera(c) => c.point,
            Intersection::Light(l) => l.point,
            Intersection::Object(o) => o.point,
        }
    }

    pub fn distance(&self) -> f64 {
        match self {
            Intersection::Camera(i) => i.direction.len(),
            Intersection::Light(i) => i.direction.len(),
            Intersection::Object(i) => i.direction.len(),
        }
    }

    pub fn is_camera(&self) -> bool {
        match self {
            Intersection::Camera(_) => true,
            _ => false,
        }
    }

    pub fn is_light(&self) -> bool {
        match self {
            Intersection::Light(_) => true,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Intersection::Object(_) => true,
            _ => false,
        }
    }
}
