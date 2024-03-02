use crate::util;
use rand::{thread_rng, Rng, RngCore};
use std::ops::Range;

pub trait Sampler {
    fn start_stream(&mut self, index: usize);
    fn sample(&mut self, range: Range<f64>) -> f64;
}

pub struct MmltSampler {
    pub large_step_probability: f64,
    sigma: f64,
    stream_count: usize,
    stream_index: usize,
    sample_index: usize,
    samples: Vec<Sample>,
    iteration: u64,
    large_step_at: u64,
    large_step: bool,
    rng: Box<dyn RngCore>,
}

struct Sample {
    value: f64,
    backup_value: f64,
    iteration: u64,
    backup_iteration: u64,
    modified_at: u64,
}

impl Sample {
    fn new(value: f64) -> Sample {
        Sample {
            value,
            backup_value: value,
            iteration: 0,
            backup_iteration: 0,
            modified_at: 0,
        }
    }

    fn backup(&mut self) {
        self.backup_value = self.value;
        self.backup_iteration = self.iteration;
    }

    fn restore(&mut self) {
        self.value = self.backup_value;
        self.iteration = self.backup_iteration;
    }
}

pub enum MutationType {
    LargeStep,
    SmallStep,
}

impl MmltSampler {
    pub fn new(stream_count: usize) -> MmltSampler {
        MmltSampler {
            large_step_probability: 0.3,
            sigma: 0.01,
            stream_count,
            stream_index: 0,
            sample_index: 0,
            samples: Vec::new(),
            iteration: 0,
            large_step_at: 0,
            large_step: false,
            rng: Box::new(thread_rng()),
        }
    }

    pub fn mutate(&mut self) -> MutationType {
        self.iteration = self.iteration + 1;
        let r = self.rng.gen_range(0.0..1.0);
        self.large_step = r < self.large_step_probability;
        if self.large_step {
            MutationType::LargeStep
        } else {
            MutationType::SmallStep
        }
    }

    pub fn accept(&mut self) {
        if self.large_step {
            self.large_step_at = self.iteration;
        }
    }

    pub fn reject(&mut self) {
        for sample in &mut self.samples {
            if sample.modified_at == self.iteration {
                sample.restore();
            }
            self.iteration = self.iteration - 1;
        }
    }
}

impl Sampler for MmltSampler {
    fn start_stream(&mut self, index: usize) {
        if index >= self.stream_count {
            panic!("invalid stream index")
        }
        self.stream_index = index;
        self.sample_index = 0;
    }

    fn sample(&mut self, range: Range<f64>) -> f64 {
        let index = self.stream_count * self.sample_index + self.stream_index;

        if index >= self.samples.len() {
            let value = self.rng.gen_range(0.0..1.0);
            let sample = Sample::new(value);
            self.samples.push(sample);
            return value;
        }

        let sample = &mut self.samples[index];

        if sample.modified_at < self.large_step_at {
            sample.value = self.rng.gen_range(0.0..1.0);
            sample.modified_at = self.large_step_at;
        }

        sample.backup();

        if self.large_step {
            sample.value = self.rng.gen_range(0.0..1.0);
        } else {
            let n = (self.iteration - sample.modified_at) as f64;
            let normal_value =
                f64::sqrt(2.0) * util::erf_inv(2.0 * self.rng.gen_range(0.0..1.0) - 1.0);
            let effective_sigma = self.sigma * n.sqrt();
            sample.value = sample.value + normal_value * effective_sigma;
            sample.value = sample.value - sample.value.floor();
        }

        sample.modified_at = self.iteration;

        self.sample_index = self.sample_index + 1;

        sample.value * (range.end - range.start) + range.start
    }
}

#[cfg(test)]
pub mod test {
    use super::Sampler;
    use std::{collections::VecDeque, ops::Range};

    pub struct MockSampler {
        samples: VecDeque<f64>,
    }

    impl MockSampler {
        pub fn new() -> MockSampler {
            MockSampler {
                samples: VecDeque::new(),
            }
        }

        pub fn add(&mut self, sample: f64) {
            self.samples.push_back(sample)
        }
    }

    impl Sampler for MockSampler {
        fn start_stream(&mut self, _index: usize) {
            // nothing
        }

        fn sample(&mut self, range: Range<f64>) -> f64 {
            let r = self.samples.pop_front().unwrap_or(0.0);
            r * (range.end - range.start) + range.start
        }
    }
}
