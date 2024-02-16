use std::ops::Range;

pub trait Sampler {
    fn start_iteration(&mut self);
    fn start_stream(&mut self, index: usize);
    fn sample(&mut self, range: Range<f64>) -> f64;
}
