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

impl PixelCoordinates {
    pub fn new(x: usize, y: usize) -> PixelCoordinates {
        PixelCoordinates { x, y }
    }
}

pub struct Image {
    pixels: Vec<Spectrum>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn configure(config: &ImageConfig) -> Image {
        Image::new(config.width, config.height)
    }

    pub fn new(width: usize, height: usize) -> Image {
        Image {
            pixels: vec![Spectrum::black(); width * height],
            width,
            height,
        }
    }

    pub fn contribute(&mut self, spectrum: Spectrum, coordinates: PixelCoordinates) {
        let i = coordinates.y * self.height + coordinates.x;
        self.pixels[i] = self.pixels[i] + spectrum;
    }

    pub fn write(&self, path: String) -> Result<(), String> {
        let m = |e: io::Error| e.to_string();
        let file = File::create(path).map_err(m)?;
        let mut writer = LineWriter::new(file);
        writeln!(writer, "PF").map_err(m)?;
        writeln!(writer, "{} {}", self.width, self.height).map_err(m)?;
        writeln!(writer, "-1.0").map_err(m)?;
        for pixel in &self.pixels {
            let rgb = pixel.to_rgb();
            writeln!(writer, "{} {} {}", rgb.r, rgb.g, rgb.b).map_err(m)?;
        }
        writer.flush().map_err(m)?;
        Ok(())
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
