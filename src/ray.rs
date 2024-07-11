use crate::vector::{Point3, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3) -> Ray {
        Ray {
            origin,
            direction: direction.norm(),
        }
    }
}
