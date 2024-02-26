use std::{
    fs::File,
    io::{self, LineWriter, Write},
};

use serde::{Deserialize, Serialize};

use crate::spectrum::Spectrum;

#[derive(Copy, Clone)]
pub struct PixelCoordinates {
    pub x: usize,
    pub y: usize,
}

pub struct Image {
    pixels: Vec<Spectrum>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new() -> Image {
        Image {
            pixels: vec![Spectrum::black(); 512 * 512],
            width: 512,
            height: 512,
        }
    }

    pub fn contribute(&mut self, spectrum: Spectrum, coordinates: PixelCoordinates) {
        let i = coordinates.y * self.height + coordinates.x;
        self.pixels[i] = self.pixels[i] + spectrum;
    }

    pub fn write(&self, path: String) -> Result<(), String> {
        let w = || {
            let file = File::create(path)?;
            let mut writer = LineWriter::new(file);
            writeln!(writer, "PF")?;
            writeln!(writer, "{} {}", self.width, self.height)?;
            writeln!(writer, "-1.0")?;
            for pixel in &self.pixels {
                let rgb = pixel.to_rgb();
                writeln!(writer, "{} {} {}", rgb.r, rgb.g, rgb.b)?;
            }
            writer.flush()?;
            Ok(())
        };

        let result = w();
        result.map_err(|e: io::Error| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageConfig {
    pub width: usize,
    pub height: usize,
    pub filter: FilterConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FilterConfig {
    Gaussian(GaussianFilterConfig),
    Box,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GaussianFilterConfig {
    radius: f64,
    alpha: f64,
}
