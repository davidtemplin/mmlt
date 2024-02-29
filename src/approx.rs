pub trait ApproxEq {
    fn approx_eq(&self, other: Self, tolerance: f64) -> bool;
}
