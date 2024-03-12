use std::env;

use crate::{
    config::Config,
    integrator::{Integrator, MmltIntegrator},
    scene::Scene,
};

mod approx;
mod bsdf;
mod camera;
mod config;
mod geometry;
mod image;
mod integrator;
mod interaction;
mod light;
mod material;
mod object;
mod path;
mod pdf;
mod ray;
mod sampler;
mod scene;
mod shape;
mod spectrum;
mod texture;
mod util;
mod vector;

fn main() {
    if let Err(e) = execute() {
        eprintln!("An error occurred: {e}");
    }
}

fn execute() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = Config::parse(args)?;
    let integrator = MmltIntegrator::new();
    let scene = Scene::load(String::from(config.scene_path))?;
    let mut image = integrator.integrate(&scene);
    image.write(config.image_path)?;
    image.tone_map();
    image.write(String::from("/Users/david/Desktop/image-tone-mapped.pfm"))
}
