use std::f64::consts::PI;

use crate::{sampler::Sampler, vector::Vector};

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
    let x = x.clamp(-0.99999, 0.99999);
    let mut w = -f64::ln((1.0 - x) * (1.0 + x));
    if w < 5.0 {
        w = w - 2.5;
        let mut p = 2.81022636e-08;
        p = 3.43273939e-07 + p * w;
        p = -3.5233877e-06 + p * w;
        p = -4.39150654e-06 + p * w;
        p = 0.00021858087 + p * w;
        p = -0.00125372503 + p * w;
        p = -0.00417768164 + p * w;
        p = 0.246640727 + p * w;
        p = 1.50140941 + p * w;
        p * x
    } else {
        w = f64::sqrt(w) - 3.0;
        let mut p = -0.000200214257;
        p = 0.000100950558 + p * w;
        p = 0.00134934322 + p * w;
        p = -0.00367342844 + p * w;
        p = 0.00573950773 + p * w;
        p = -0.0076224613 + p * w;
        p = 0.00943887047 + p * w;
        p = 1.00167406 + p * w;
        p = 2.83297682 + p * w;
        p * x
    }
}

pub fn concentric_sample_disk(sampler: &mut dyn Sampler) -> (f64, f64) {
    let u1 = sampler.sample(0.0..1.0);
    let u2 = sampler.sample(0.0..1.0);

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

pub fn cosine_sample_hemisphere(n: Vector, sampler: &mut dyn Sampler) -> Vector {
    // Sample a unit disk in R^2
    let (x, y) = concentric_sample_disk(sampler);

    // Compute an orthonormal basis relative to n as the "z" direction
    let (nx, ny, nz) = orthonormal_basis(n);

    // Compute the coordinates in this new orthonormal basis relative to the normal vector nz
    let z = f64::max(0.0, 1.0 - x * x - y * y).sqrt();

    nx * x + ny * y + nz * z
}

pub fn orthonormal_basis(n: Vector) -> (Vector, Vector, Vector) {
    let nz = n.norm();
    let ey = Vector::new(0.0, 1.0, 0.0);
    let mut nx = nz.cross(ey).norm();
    let ny = if nx.is_zero() {
        let ex = Vector::new(1.0, 0.0, 0.0);
        let ny = ex.cross(nz).norm();
        nx = nz.cross(ny).norm();
        ny
    } else {
        nx.cross(nz).norm()
    };
    (nx, ny, nz)
}

pub fn same_hemisphere(n: Vector, v1: Vector, v2: Vector) -> bool {
    v1.dot(n).is_sign_positive() == v2.dot(n).is_sign_positive()
}

pub fn abs_cos_theta(n: Vector, v: Vector) -> f64 {
    n.norm().dot(v.norm()).abs()
}

pub fn uniform_sample_sphere(sampler: &mut dyn Sampler) -> Vector {
    let u1 = sampler.sample(0.0..1.0);
    let u2 = sampler.sample(0.0..1.0);
    let z = 1.0 - 2.0 * u1;
    let r = f64::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u2;
    Vector::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn equals(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

pub fn reflect(d: Vector, n: Vector) -> Vector {
    d - (2.0 * d.dot(n) * n)
}
