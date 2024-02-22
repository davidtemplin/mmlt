use std::f64::consts::PI;

use crate::vector::Vector;

pub fn direction_to_area(direction: Vector, normal: Vector) -> f64 {
    let d2 = direction.dot(direction);
    let x = normal.dot(direction) / (d2 * d2.sqrt());
    x.abs()
}

pub fn geometry_term(direction: Vector, normal1: Vector, normal2: Vector) -> f64 {
    let d2 = direction.dot(direction);
    let x = (normal1.dot(direction) * normal2.dot(direction)) / (d2 * d2);
    x.abs()
}

pub fn erf_inv(x: f64) -> f64 {
    let mut w = 0.0;
    let mut p = 0.0;
    let x = x.clamp(-0.99999, 0.99999);
    w = -f64::ln((1.0 - x) * (1.0 + x));
    if w < 5.0 {
        w = w - 2.5;
        p = 2.81022636e-08;
        p = 3.43273939e-07 + p * w;
        p = -3.5233877e-06 + p * w;
        p = -4.39150654e-06 + p * w;
        p = 0.00021858087 + p * w;
        p = -0.00125372503 + p * w;
        p = -0.00417768164 + p * w;
        p = 0.246640727 + p * w;
        p = 1.50140941 + p * w;
    } else {
        w = f64::sqrt(w) - 3.0;
        p = -0.000200214257;
        p = 0.000100950558 + p * w;
        p = 0.00134934322 + p * w;
        p = -0.00367342844 + p * w;
        p = 0.00573950773 + p * w;
        p = -0.0076224613 + p * w;
        p = 0.00943887047 + p * w;
        p = 1.00167406 + p * w;
        p = 2.83297682 + p * w;
    }
    return p * x;
}

pub fn concentric_sample_disk(u1: f64, u2: f64) -> (f64, f64) {
    // Map uniform random numbers to $[-1,1]^2$
    let u_offset_x = 2.0 * u1 - 1.0;
    let u_offset_y = 2.0 * u2 - 1.0;

    // Handle degeneracy at the origin
    if u_offset_x == 0.0 && u_offset_y == 0.0 {
        return (0.0, 0.0);
    }

    // Apply concentric mapping to point
    let (theta, r) = if u_offset_x.abs() > u_offset_y.abs() {
        (u_offset_x, (PI / 4.0) * (u_offset_y / u_offset_x))
    } else {
        (
            u_offset_y,
            (PI / 2.0 - PI / 4.0) * (u_offset_x / u_offset_y),
        )
    };

    // Done
    (r * theta.cos(), r * theta.sin())
}

pub fn cosine_sample_hemisphere(u1: f64, u2: f64) -> Vector {
    let (x, y) = concentric_sample_disk(u1, u2);
    let z = f64::max(0.0, 1.0 - x * x - y * y).sqrt();
    Vector::new(x, y, z)
}

pub fn same_hemisphere(w: Vector, wp: Vector) -> bool {
    w.z * wp.z > 0.0
}

pub fn abs_cos_theta(v: Vector) -> f64 {
    v.z
}

pub fn uniform_sample_sphere(u1: f64, u2: f64) -> Vector {
    let z = 1.0 - 2.0 * u1;
    let r = f64::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u2;
    Vector::new(r * phi.cos(), r * phi.sin(), z)
}
