use rand::{distributions::Distribution, thread_rng, Rng};

use crate::{
    config::Config,
    image::Image,
    path::{Contribution, Path},
    pdf::Pdf,
    progress::{report, report_progress},
    sampler::{MmltSampler, MutationType},
    scene::Scene,
};

pub trait Integrator {
    fn integrate(&self, scene: &Scene) -> Image;
}

pub struct MmltIntegrator {
    max_path_length: usize,
    initial_sample_count: u64,
    average_samples_per_pixel: u64,
}

impl MmltIntegrator {
    pub fn new(config: &Config) -> MmltIntegrator {
        MmltIntegrator {
            max_path_length: config.max_path_length.unwrap_or(20),
            initial_sample_count: config.initial_sample_count.unwrap_or(100_000),
            average_samples_per_pixel: config.average_samples_per_pixel.unwrap_or(4096),
        }
    }
}

impl Integrator for MmltIntegrator {
    fn integrate(&self, scene: &Scene) -> Image {
        report("Initializing MMLT integrator...");

        let mut b = vec![0.0; self.max_path_length - 1];
        let mut rng = thread_rng();

        for k in 0..self.max_path_length - 1 {
            for _ in 0..self.initial_sample_count {
                let mut sampler = Path::sampler();
                let contribution = Path::contribute(scene, &mut sampler, k + 2);
                b[k] = b[k] + contribution.scalar;
            }
            b[k] = b[k] / self.initial_sample_count as f64;
            report_progress((k + 1) as f64 / (self.max_path_length - 1) as f64);
        }

        let pdf = Pdf::new(&b);
        let mut samplers: Vec<MmltSampler> = Vec::new();
        let mut contributions: Vec<Contribution> = Vec::new();

        for k in 0..self.max_path_length - 1 {
            let mut sampler = Path::sampler();
            let contribution = Path::contribute(scene, &mut sampler, k + 2);
            contributions.push(contribution);
            samplers.push(sampler);
        }

        let mut sample_count: u64 = 0;
        let mut image = Image::configure(&scene.image_config);
        let pixel_count = (scene.image_config.width * scene.image_config.height) as u64;
        let mut spp = 0;
        let mut last_reported_spp = 0;

        report("Integrating...");

        while spp < self.average_samples_per_pixel {
            spp = sample_count / pixel_count;
            if last_reported_spp < spp {
                report_progress(spp as f64 / self.average_samples_per_pixel as f64);
                last_reported_spp = spp;
            }
            sample_count = sample_count + 1;
            let k = pdf.sample(&mut rng);
            let sampler = &mut samplers[k];
            let mutation_type = sampler.mutate();
            let current_contribution = contributions[k];
            let proposal_contribution = Path::contribute(scene, sampler, k + 2);
            let a = Contribution::acceptance(current_contribution, proposal_contribution);
            let step_factor = match mutation_type {
                MutationType::LargeStep => 1.0,
                MutationType::SmallStep => 0.0,
            };

            if !proposal_contribution.is_empty() {
                let weight = (((k as f64 + 2.0) / pdf.value(k)) * (a + step_factor))
                    / ((proposal_contribution.scalar / b[k]) + sampler.large_step_probability);
                let spectrum = proposal_contribution.spectrum * weight;
                image.contribute(spectrum, proposal_contribution.pixel_coordinates);
            }

            if !current_contribution.is_empty() {
                let weight = (((k as f64 + 2.0) / pdf.value(k)) * (1.0 - a))
                    / ((current_contribution.scalar / b[k]) + sampler.large_step_probability);
                let spectrum = current_contribution.spectrum * weight;
                image.contribute(spectrum, current_contribution.pixel_coordinates);
            }

            if rng.gen_range(0.0..1.0) <= a {
                sampler.accept();
                contributions[k] = proposal_contribution;
            } else {
                sampler.reject();
            }
        }

        image.scale(1.0 / self.average_samples_per_pixel as f64);

        report("MMLT integration complete");

        image
    }
}
