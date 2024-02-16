use std::{
    env,
    error::Error,
    io::{self, Stderr},
};

use crate::{
    config::Config,
    integrator::{Integrator, MmltIntegrator},
    scene::Scene,
};

mod camera;
mod config;
mod image;
mod integrator;
mod intersection;
mod light;
mod markov_chain;
mod object;
mod path;
mod pdf;
mod ray;
mod sampler;
mod scene;
mod shape;
mod spectrum;
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
    let scene = Scene::load(config.scene_path)?;
    let image = integrator.integrate(&scene);
    image.write(config.image_path)
}
