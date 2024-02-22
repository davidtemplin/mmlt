use crate::vector::{Point, Vector};

#[derive(Copy, Clone)]
pub struct Geometry {
    pub point: Point,
    pub normal: Vector,
    pub direction: Vector,
}
