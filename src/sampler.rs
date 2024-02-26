use crate::util;
use rand::{thread_rng, Rng};
use std::ops::Range;

pub trait Sampler {
    fn start_iteration(&mut self);
    fn start_stream(&mut self, index: usize);
    fn sample(&mut self, range: Range<f64>) -> f64;
}

pub struct MmltSampler {
    pub large_step_probability: f64,
    sigma: f64,
    stream_count: usize,
    stream_index: usize,
    sample_index: usize,
    state: Vec<Sample>,
    iteration: u64,
    large_step_at: u64,
    large_step: bool,
}

struct Sample {
    value: f64,
    backup: f64,
    iteration: u64,
    backup_iteration: u64,
    modified_at: u64,
}

impl Sample {
    fn new(value: f64) -> Sample {
        Sample {
            value,
            backup: value,
            iteration: 0,
            backup_iteration: 0,
            modified_at: 0,
        }
    }

    fn backup(&mut self) {
        self.backup = self.value;
        self.backup_iteration = self.iteration;
    }

    fn restore(&mut self) {
        self.value = self.backup;
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
            state: Vec::new(),
            iteration: 0,
            large_step_at: 0,
            large_step: false,
        }
    }

    pub fn mutate(&mut self) -> MutationType {
        let rng = &mut thread_rng();
        let r = rng.gen_range(0.0..1.0);
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
        for sample in &mut self.state {
            if sample.modified_at == self.iteration {
                sample.restore();
            }
            self.iteration = self.iteration - 1;
        }
    }
}

impl Sampler for MmltSampler {
    fn start_iteration(&mut self) {
        self.iteration = self.iteration + 1;
    }

    fn start_stream(&mut self, index: usize) {
        if index >= self.stream_count {
            panic!("invalid stream index")
        }
        self.stream_index = index;
    }

    fn sample(&mut self, range: Range<f64>) -> f64 {
        let index = self.stream_count * self.sample_index + self.stream_index;

        let mut rng = thread_rng();

        if index >= self.state.len() {
            let value = rng.gen_range(0.0..1.0);
            let sample = Sample::new(value);
            self.state.push(sample);
            return value;
        }

        let sample = &mut self.state[index];

        if sample.modified_at < self.large_step_at {
            sample.value = rng.gen_range(0.0..1.0);
            sample.modified_at = self.large_step_at;
        }

        sample.backup();

        if self.large_step {
            sample.value = rng.gen_range(0.0..1.0);
        } else {
            let n = f64::from((self.iteration - sample.modified_at) as i32);
            let normal_value = f64::sqrt(2.0) * util::erf_inv(2.0 * rng.gen_range(0.0..1.0) - 1.0);
            let effective_sigma = self.sigma * n.sqrt();
            sample.value = sample.value + normal_value * effective_sigma;
            sample.value = sample.value - sample.value.floor();
        }

        sample.modified_at = self.iteration;

        sample.value * (range.start - range.end) + range.start
    }
}
