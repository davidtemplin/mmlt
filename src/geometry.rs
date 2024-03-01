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

#[cfg(test)]
mod tests {
    use super::Geometry;
    use crate::{
        approx::ApproxEq,
        vector::{Point, Vector},
    };

    #[test]
    fn test_geometry_eq() {
        let g1 = Geometry {
            point: Point::new(1.0, 1.0, 1.0),
            normal: Vector::new(1.0, 0.0, 0.0),
            direction: Vector::new(1.0, 1.0, 1.0),
        };

        assert_eq!(g1, g1);
    }

    #[test]
    fn test_geometry_approx_eq() {
        let g1 = Geometry {
            point: Point::new(1.0, 1.0, 1.0),
            normal: Vector::new(1.0, 0.0, 0.0),
            direction: Vector::new(1.0, 1.0, 1.0),
        };

        let g2 = Geometry {
            point: g1.point + Point::new(1e-9, 1e-9, 1e-9),
            normal: g1.normal + Vector::new(1e-9, 1e-9, 1e-9),
            direction: g1.direction + Vector::new(1e-9, 1e-9, 1e-9),
        };

        assert!(g1.approx_eq(g2, 1e-8));
    }
}
