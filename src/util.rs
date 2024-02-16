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
