use crate::{
    approx::ApproxEq,
    vector::{Point3, Vector3},
};

#[derive(Copy, Clone, Debug)]
pub struct Geometry {
    pub point: Point3,
    pub normal: Vector3,
    pub direction: Vector3,
}

impl Geometry {
    pub fn set_direction(&mut self, direction: Vector3) {
        self.direction = direction;
    }
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
        vector::{Point3, Vector3},
    };

    #[test]
    fn test_geometry_eq() {
        let g1 = Geometry {
            point: Point3::new(1.0, 1.0, 1.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            direction: Vector3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(g1, g1);
    }

    #[test]
    fn test_geometry_approx_eq() {
        let g1 = Geometry {
            point: Point3::new(1.0, 1.0, 1.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            direction: Vector3::new(1.0, 1.0, 1.0),
        };

        let g2 = Geometry {
            point: g1.point + Point3::new(1e-9, 1e-9, 1e-9),
            normal: g1.normal + Vector3::new(1e-9, 1e-9, 1e-9),
            direction: g1.direction + Vector3::new(1e-9, 1e-9, 1e-9),
        };

        assert!(g1.approx_eq(g2, 1e-8));
    }
}
