use std::{
    fs::File,
    io::{self, LineWriter, Write},
};

use serde::{Deserialize, Serialize};

use crate::spectrum::Spectrum;

#[derive(Copy, Clone, Debug)]
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
        let i = coordinates.y * self.width + coordinates.x;
        self.pixels[i] = self.pixels[i] + spectrum;
    }

    pub fn write(&self, path: String) -> Result<(), String> {
        let m = |e: io::Error| e.to_string();
        let file = File::create(path).map_err(m)?;
        let mut writer = LineWriter::new(file);
        writeln!(writer, "PF").map_err(m)?;
        writeln!(writer, "{} {}", self.width, self.height).map_err(m)?;
        writeln!(writer, "-1").map_err(m)?;
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let i = (y * self.width + x) as usize;
                let pixel = self.pixels[i];
                let rgb = pixel.to_rgb();
                writer.write(&(rgb.r as f32).to_le_bytes()).map_err(m)?;
                writer.write(&(rgb.g as f32).to_le_bytes()).map_err(m)?;
                writer.write(&(rgb.b as f32).to_le_bytes()).map_err(m)?;
            }
        }
        writer.flush().map_err(m)?;
        Ok(())
    }

    pub fn scale(&mut self, s: f64) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.pixels[i] * s;
        }
    }

    pub fn tone_map(&mut self) {
        let min = self
            .pixels
            .iter()
            .map(|p| p.luminance())
            .min_by(f64::total_cmp)
            .unwrap();

        let max = self
            .pixels
            .iter()
            .map(|p| p.luminance())
            .max_by(f64::total_cmp)
            .unwrap();

        println!("min = {}, max = {}", min, max);

        for i in 0..self.pixels.len() {
            let l_in = self.pixels[i].luminance();
            let l_out = 65536.0 * (l_in - min) / (max - min);
            let scale = l_out / l_in;
            self.pixels[i] = self.pixels[i] * scale;
        }
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
