use std::{
    fs::File,
    io::{self, LineWriter, Write},
};

use exr::image::write::write_rgb_file;
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
        if !spectrum.has_nans() {
            self.pixels[i] = self.pixels[i] + spectrum;
        } else {
            eprintln!("warning: NaN detected");
        }
    }

    pub fn write(&self, path: String) -> Result<(), String> {
        if path.ends_with(".pfm") {
            self.write_pfm(path)
        } else if path.ends_with(".exr") {
            self.write_exr(path)
        } else {
            Err(String::from("unknown image type"))
        }
    }

    fn write_pfm(&self, path: String) -> Result<(), String> {
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

    fn write_exr(&self, path: String) -> Result<(), String> {
        write_rgb_file(path, self.width, self.height, |x, y| {
            let i = y * self.width + x;
            let pixel = self.pixels[i];
            let rgb = pixel.to_rgb();
            (rgb.r as f32, rgb.g as f32, rgb.b as f32)
        })
        .map_err(|e| e.to_string())
    }

    pub fn scale(&mut self, s: f64) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.pixels[i] * s;
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
