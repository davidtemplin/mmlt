use crate::{
    approx::ApproxEq,
    vector::{Point, Vector},
};

#[derive(Copy, Clone, Debug)]
pub struct Geometry {
    pub point: Point,
    pub normal: Vector,
    pub direction: Vector,
}

impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
            && self.normal == other.normal
            && self.direction == other.direction
    }
}

impl ApproxEq for Geometry {
    fn approx_eq(&self, other: Self, tolerance: f64) -> bool {
        self.point.approx_eq(other.point, tolerance)
            && self.normal.approx_eq(other.normal, tolerance)
            && self.direction.approx_eq(other.direction, tolerance)
    }
}
