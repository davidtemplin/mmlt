use std::{env, io};

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

fn main() -> io::Result<()> {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let config = Config::parse(args);
    let integrator = MmltIntegrator::new();
    let scene = Scene::new();
    let image = integrator.integrate(&scene);
    image.write("foo.image")?;
    Ok(())
}
