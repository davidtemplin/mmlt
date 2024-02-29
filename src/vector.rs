use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use serde::Deserialize;
use serde::Serialize;

use crate::approx::ApproxEq;
use crate::util;

pub type Point = Vector;

#[derive(Copy, Clone, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn configure(config: &VectorConfig) -> Vector {
        Vector {
            x: config.x,
            y: config.y,
            z: config.z,
        }
    }

    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    pub fn dot(&self, rhs: Vector) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn norm(&self) -> Vector {
        let l = self.len();
        if l == 0.0 {
            *self
        } else {
            *self / self.len()
        }
    }

    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn cross(&self, rhs: Vector) -> Vector {
        Vector {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Vector {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Vector {
        Vector {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl ApproxEq for Vector {
    fn approx_eq(&self, other: Self, tolerance: f64) -> bool {
        util::equals(self.x, other.x, tolerance)
            && util::equals(self.y, other.y, tolerance)
            && util::equals(self.z, other.z, tolerance)
    }
}

pub type PointConfig = VectorConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorConfig {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[cfg(test)]
mod tests {
    use crate::vector::Vector;

    use super::VectorConfig;

    #[test]
    fn test_configure() {
        let config = VectorConfig {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let v = Vector::configure(&config);
        assert_eq!(v, Vector::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_new() {
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_dot() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(v1.dot(v2), 20.0);
    }

    #[test]
    fn test_norm() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let l1 = f64::sqrt(14.0);
        assert_eq!(v1.norm(), Vector::new(1.0 / l1, 2.0 / l1, 3.0 / l1));
        let v2 = Vector::new(0.0, 0.0, 2.0);
        assert_eq!(v2.norm(), Vector::new(0.0, 0.0, 1.0));
        let v3 = Vector::new(0.0, 0.0, 0.0);
        assert_eq!(v3.norm(), v3);
    }

    #[test]
    fn test_len() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v1.len(), f64::sqrt(14.0));
    }

    #[test]
    fn test_cross() {
        let v1 = Vector::new(0.0, 0.0, 1.0);
        let v2 = Vector::new(1.0, 0.0, 0.0);
        assert_eq!(v1.cross(v2), Vector::new(0.0, 1.0, 0.0));
        assert_eq!(v2.cross(v1), Vector::new(0.0, -1.0, 0.0));
        let v3 = Vector::new(1.0, 2.0, -3.0);
        let v4 = Vector::new(0.0, -4.0, 3.0);
        assert_eq!(v3.cross(v4), -v4.cross(v3));
    }

    #[test]
    fn test_is_zero() {
        let v1 = Vector::new(1e-8, 0.0, 0.0);
        assert!(!v1.is_zero());
        let v2 = Vector::new(0.0, 0.0, 0.0);
        assert!(v2.is_zero());
    }

    #[test]
    fn test_add() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(v1 + v2, Vector::new(3.0, 5.0, 7.0));
    }

    #[test]
    fn test_sub() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(v1 - v2, Vector::new(-1.0, -1.0, -1.0));
    }

    #[test]
    fn test_mul() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v1 * 2.0, Vector::new(2.0, 4.0, 6.0));
        assert_eq!(-2.0 * v1, Vector::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn test_div() {
        let v1 = Vector::new(1.0, 2.0, -4.0);
        assert_eq!(v1 / 2.0, Vector::new(0.5, 1.0, -2.0));
    }

    #[test]
    fn test_neg() {
        let v1 = Vector::new(1.0, -2.0, 3.0);
        assert_eq!(-v1, Vector::new(-1.0, 2.0, -3.0));
    }

    #[test]
    fn test_eq() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(2.0, 3.0, 4.0);
        assert!(v1 == v1);
        assert!(v1 != v2);
    }
}
