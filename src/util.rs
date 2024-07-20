use std::f64::consts::PI;

use crate::{sampler::Sampler, vector::Vector3};

pub fn direction_to_area(direction: Vector3, normal: Vector3) -> f64 {
    let d2 = direction.dot(direction);
    let x = normal.dot(direction) / (d2 * d2.sqrt());
    x.abs()
}

pub fn geometry_term(direction: Vector3, normal1: Vector3, normal2: Vector3) -> f64 {
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
    let (r, theta) = if u_offset_x.abs() > u_offset_y.abs() {
        (u_offset_x, (PI / 4.0) * (u_offset_y / u_offset_x))
    } else {
        (
            u_offset_y,
            (PI / 2.0) - ((PI / 4.0) * (u_offset_x / u_offset_y)),
        )
    };

    // Done
    (r * theta.cos(), r * theta.sin())
}

pub fn cosine_sample_hemisphere(n: Vector3, sampler: &mut dyn Sampler) -> Vector3 {
    // Sample a unit disk in R^2
    let (x, y) = concentric_sample_disk(sampler);

    // Compute an orthonormal basis relative to n as the "z" direction
    let (nx, ny, nz) = orthonormal_basis(n);

    // Compute the coordinates in this new orthonormal basis relative to the normal vector nz
    let z = f64::max(0.0, 1.0 - x * x - y * y).sqrt();

    nx * x + ny * y + nz * z
}

pub fn orthonormal_basis(n: Vector3) -> (Vector3, Vector3, Vector3) {
    let nz = n.norm();
    let ey = Vector3::new(0.0, 1.0, 0.0);
    let mut nx = ey.cross(nz).norm();
    let ny = if nx.is_zero() {
        let ex = Vector3::new(1.0, 0.0, 0.0);
        let ny = nz.cross(ex).norm();
        nx = ny.cross(nz).norm();
        ny
    } else {
        nz.cross(nx).norm()
    };
    (nx, ny, nz)
}

pub fn same_hemisphere(n: Vector3, v1: Vector3, v2: Vector3) -> bool {
    v1.dot(n).is_sign_positive() == v2.dot(n).is_sign_positive()
}

pub fn abs_cos_theta(n: Vector3, v: Vector3) -> f64 {
    n.norm().dot(v.norm()).abs()
}

pub fn uniform_sample_sphere(sampler: &mut dyn Sampler) -> Vector3 {
    let u1 = sampler.sample(0.0..1.0);
    let u2 = sampler.sample(0.0..1.0);
    let z = 1.0 - 2.0 * u1;
    let r = f64::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u2;
    Vector3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn equals(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

pub fn reflect(d: Vector3, n: Vector3) -> Vector3 {
    -d + (2.0 * d.dot(n) * n)
}

pub fn gaussian(x: f64, sigma: f64) -> f64 {
    f64::exp(-(x * x) / (2.0 * sigma * sigma))
}

pub fn safe_sqrt(x: f64) -> f64 {
    f64::max(0.0, x).sqrt()
}

pub fn refract(wi: Vector3, mut n: Vector3, mut eta: f64) -> Option<Vector3> {
    let mut cos_theta_i = cos_theta(n, wi);

    if cos_theta_i < 0.0 {
        eta = 1.0 / eta;
        cos_theta_i = -cos_theta_i;
        n = -n;
    }

    let sin2_theta_i = f64::max(0.0, 1.0 - sqr(cos_theta_i));
    let sin2_theta_t = sin2_theta_i / sqr(eta);
    if sin2_theta_t >= 1.0 {
        return None;
    }

    let cos_theta_t = safe_sqrt(1.0 - sin2_theta_t);

    let wt = -wi / eta + (cos_theta_i / eta - cos_theta_t) * n;

    Some(wt)
}

pub fn sqr(x: f64) -> f64 {
    x * x
}

pub fn cos_theta(a: Vector3, b: Vector3) -> f64 {
    a.norm().dot(b.norm())
}

pub fn fresnel_dielectric(mut cos_theta_i: f64, mut eta: f64) -> f64 {
    cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);

    if cos_theta_i < 0.0 {
        eta = 1.0 / eta;
        cos_theta_i = -cos_theta_i;
    }

    let sin2_theta_i = 1.0 - sqr(cos_theta_i);
    let sin2_theta_t = sin2_theta_i / sqr(eta);
    if sin2_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = safe_sqrt(1.0 - sin2_theta_t);

    let r_parallel = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);

    let r_perpendicular = (cos_theta_i - eta * cos_theta_t) / (cos_theta_i + eta * cos_theta_t);

    (sqr(r_parallel) + sqr(r_perpendicular)) / 2.0
}

#[cfg(test)]
mod tests {
    use super::{
        concentric_sample_disk, cosine_sample_hemisphere, direction_to_area, erf_inv,
        geometry_term, orthonormal_basis, reflect, refract,
    };
    use crate::{approx::ApproxEq, sampler::test::MockSampler, vector::Vector3};
    use std::f64::consts::PI;

    #[test]
    fn test_orthonormal_basis() {
        let d1 = Vector3::new(0.0, 0.0, 2.0);
        let (u1, v1, w1) = orthonormal_basis(d1);
        assert_eq!(u1, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(v1, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(w1, Vector3::new(0.0, 0.0, 1.0));

        let d2 = Vector3::new(2.0, 0.0, 0.0);
        let (u2, v2, w2) = orthonormal_basis(d2);
        assert_eq!(u2, Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(v2, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(w2, Vector3::new(1.0, 0.0, 0.0));

        let d3 = Vector3::new(0.0, 2.0, 0.0);
        let (u3, v3, w3) = orthonormal_basis(d3);
        assert_eq!(u3, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(v3, Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(w3, Vector3::new(0.0, 1.0, 0.0));

        let d4 = Vector3::new(1.0, 1.0, 1.0);
        let (u4, v4, w4) = orthonormal_basis(d4);
        assert!((u4 - Vector3::new(1.0, 0.0, -1.0).norm()).len() < 1e-5);
        assert!((v4 - Vector3::new(-1.0, 2.0, -1.0).norm()).len() < 1e-5);
        assert!((w4 - d4.norm()).len() < 1e-5);

        let d5 = Vector3::new(0.0, -2.0, 0.0);
        let (u5, v5, w5) = orthonormal_basis(d5);
        assert_eq!(u5, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(v5, Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(w5, Vector3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn test_erf_inv() {
        assert!(erf_inv(0.5) - 0.47693628 < 2e-8);
    }

    #[test]
    fn test_direction_to_area() {
        let d = Vector3::new(10.0, 0.0, 0.0);
        let angle = PI / 4.0;
        let n = Vector3::new(-f64::cos(angle), f64::sin(angle), 0.0).norm();
        let a = direction_to_area(d, n);
        let e = (f64::cos(angle) / (d.len() * d.len())).abs();
        assert!(a - e < 1e-8);
    }

    #[test]
    fn test_geometry_term() {
        let d = Vector3::new(10.0, 0.0, 0.0);
        let angle1 = PI / 4.0;
        let angle2 = PI / 3.0;
        let n1 = Vector3::new(f64::cos(angle1), -f64::sin(angle1), 0.0).norm();
        let n2 = Vector3::new(-f64::cos(angle2), f64::sin(angle2), 0.0).norm();
        let g = geometry_term(d, n1, n2);
        let e = ((f64::cos(angle1) * f64::cos(angle2)) / (d.len() * d.len())).abs();
        assert!(g - e < 1e-8);
    }

    #[test]
    fn test_concentric_sample_disk() {
        let mut sampler = MockSampler::new();
        sampler.add(0.2);
        sampler.add(0.7);
        let (x, y) = concentric_sample_disk(&mut sampler);
        assert!(f64::sqrt(x * x + y * y) < 1.0);
    }

    #[test]
    fn test_cosine_sample_hemisphere() {
        let mut sampler = MockSampler::new();
        sampler.add(0.7);
        sampler.add(0.5);
        let n = Vector3::new(0.0, -1.0, 0.0);
        let v = cosine_sample_hemisphere(n, &mut sampler);
        let tolerance = 1.0e-5;
        assert!(1.0 - v.len() < tolerance);
        assert!(v.dot(n) > 0.0);
    }

    #[test]
    fn test_reflect() {
        let d = Vector3::new(-1.0, 1.0, 0.0);
        let n = Vector3::new(0.0, 1.0, 0.0);
        let r = reflect(d, n);
        let expected = Vector3::new(1.0, 1.0, 0.0);
        assert!((expected - r).len() < 1e-5);
    }

    #[test]
    fn test_refract() {
        let mut wi = Vector3::new(-1.0, 1.0, 0.0).norm();
        let n = Vector3::new(0.0, 1.0, 0.0);
        let mut eta = 1.0;
        let mut wt = refract(wi, n, eta);
        assert!(wt.is_some());
        let mut expected = -wi;
        assert!(wt.unwrap().approx_eq(expected, 1e-6));

        eta = 1.6;
        let mut theta_i = 30.0 * PI / 180.0;
        wi = Vector3::new(-f64::sin(theta_i), f64::cos(theta_i), 0.0);
        wt = refract(wi, n, eta);
        assert!(wt.is_some());
        let theta_t = 18.20996 * PI / 180.0;
        expected = Vector3::new(f64::sin(theta_t), -f64::cos(theta_t), 0.0);
        assert!(wt.unwrap().approx_eq(expected, 1e-6));

        eta = 1.8;
        theta_i = 20.0 * PI / 180.0;
        wi = Vector3::new(-f64::sin(theta_i), f64::cos(theta_i), 0.0);
        wt = refract(wi, n, eta);
        assert!(wt.is_some());
        let theta_t = 10.95344 * PI / 180.0;
        expected = Vector3::new(f64::sin(theta_t), -f64::cos(theta_t), 0.0);
        assert!(wt.unwrap().approx_eq(expected, 1e-6));
    }
}
