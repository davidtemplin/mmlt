use rand::{distributions::Distribution, thread_rng, Rng};

use crate::{
    image::Image,
    path::{Contribution, Path},
    pdf::Pdf,
    sampler::{MmltSampler, MutationType},
    scene::Scene,
};

pub trait Integrator {
    fn integrate(&self, scene: &Scene) -> Image;
}

pub struct MmltIntegrator {
    max_path_lenth: usize,
    initial_sample_count: u64,
    average_samples_per_pixel: u64,
}

impl MmltIntegrator {
    pub fn new() -> MmltIntegrator {
        MmltIntegrator {
            max_path_lenth: 20,
            initial_sample_count: 100_000,
            average_samples_per_pixel: 100,
        }
    }
}

impl Integrator for MmltIntegrator {
    fn integrate(&self, scene: &Scene) -> Image {
        let mut b = vec![0.0; self.max_path_lenth];
        let mut rng = thread_rng();

        for k in 1..self.max_path_lenth {
            for _ in 1..self.initial_sample_count {
                let mut sampler = Path::sampler();
                if let Some(path) = Path::generate(scene, &mut sampler, k) {
                    if let Some(contribution) = path.contribution() {
                        b[k] = b[k] + contribution.scalar;
                    }
                }
            }
        }

        let pdf = Pdf::new(&b);
        let mut samplers: Vec<MmltSampler> = Vec::new();
        let mut contributions: Vec<Contribution> = Vec::new();

        for k in 1..self.max_path_lenth {
            let mut sampler = Path::sampler();
            if let Some(path) = Path::generate(scene, &mut sampler, k) {
                if let Some(contribution) = path.contribution() {
                    samplers[k] = sampler;
                    contributions[k] = contribution;
                }
            }
        }

        let mut sample_count: u64 = 0;
        let mut image = Image::configure(&scene.image_config);
        let pixel_count = (scene.image_config.width * scene.image_config.height) as u64;

        while sample_count / pixel_count < self.average_samples_per_pixel {
            sample_count = sample_count + 1;
            let k = pdf.sample(&mut rng);
            let sampler = &mut samplers[k];
            let mutation_type = sampler.mutate();

            if let Some(path) = Path::generate(scene, sampler, k) {
                if let Some(proposal_contribution) = path.contribution() {
                    let current_contribution = contributions[k];
                    let a = proposal_contribution.ratio(current_contribution);

                    if proposal_contribution.scalar > 0.0 {
                        let step_factor = match mutation_type {
                            MutationType::LargeStep => 1.0,
                            MutationType::SmallStep => 0.0,
                        };
                        let weight = (k as f64 + 2.0) / pdf.value(k) * (a + step_factor)
                            / (proposal_contribution.scalar / b[k]
                                + sampler.large_step_probability);
                        let spectrum = proposal_contribution.spectrum * weight;
                        image.contribute(spectrum, proposal_contribution.pixel_coordinates);
                    }

                    if current_contribution.scalar > 0.0 {
                        let weight = (k as f64 + 2.0) / pdf.value(k) * (1.0 - a)
                            / (current_contribution.scalar / b[k] + sampler.large_step_probability);
                        let spectrum = current_contribution.spectrum * weight;
                        image.contribute(spectrum, current_contribution.pixel_coordinates);
                    }

                    if rng.gen_range(0.0..1.0) <= a {
                        sampler.accept();
                    } else {
                        sampler.reject();
                    }
                }
            }
        }

        image
    }
}
