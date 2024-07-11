use std::{
    fs::File,
    io::{self, LineWriter, Write},
};

use exr::image::write::write_rgb_file;
use serde::{Deserialize, Serialize};

use crate::{
    spectrum::Spectrum,
    util,
    vector::{Point2, Vector2, Vector2Config},
};

pub struct Image {
    pixels: Vec<Spectrum>,
    width: usize,
    height: usize,
    filter: Box<dyn Filter>,
    sample_clamp: Option<f64>,
    clamp: Option<f64>,
}

impl Image {
    pub fn configure(config: &ImageConfig) -> Image {
        Image::new(
            config.width,
            config.height,
            config.filter.configure(),
            config.sample_clamp,
            config.clamp,
        )
    }

    pub fn new(
        width: usize,
        height: usize,
        filter: Box<dyn Filter>,
        sample_clamp: Option<f64>,
        clamp: Option<f64>,
    ) -> Image {
        Image {
            pixels: vec![Spectrum::black(); width * height],
            width,
            height,
            filter,
            sample_clamp,
            clamp,
        }
    }

    pub fn contribute(&mut self, spectrum: Spectrum, coordinates: Point2) {
        if !spectrum.has_nans() {
            let radius = self.filter.radius();
            let min_x = usize::max(0, (coordinates.x - radius.x) as usize);
            let max_x = usize::min(self.width - 1, (coordinates.x + radius.x) as usize);
            let min_y = usize::max(0, (coordinates.y - radius.y) as usize);
            let max_y = usize::min(self.height - 1, (coordinates.y + radius.y) as usize);
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let i = y * self.width + x;
                    let p = Point2::new(x as f64, y as f64);
                    let weight = self.filter.evaluate(coordinates - p);
                    self.pixels[i] =
                        self.pixels[i] + weight * spectrum.try_clamp(self.sample_clamp);
                    self.pixels[i] = self.pixels[i].try_clamp(self.clamp);
                }
            }
        } else {
            eprintln!("warning: NaN detected");
        }
    }

    pub fn write(&self, path: String) -> Result<(), String> {
        if path.ends_with(".pfm") {
            self.write_pfm(path)
        } else if path.ends_with(".exr") {
            self.write_exr(path)
        } else if path.ends_with("ppm") {
            self.write_ppm(path)
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

    fn write_ppm(&self, path: String) -> Result<(), String> {
        let m = |e: io::Error| e.to_string();
        let file = File::create(path).map_err(m)?;
        let mut writer = LineWriter::new(file);
        writeln!(writer, "P6").map_err(m)?;
        writeln!(writer, "{} {}", self.width, self.height).map_err(m)?;
        writeln!(writer, "255").map_err(m)?;
        let correct = |value: f64| -> [u8; 1] {
            let tone_mapped_value = 1.0 - f64::exp(-value);
            let gamma_corrected_value = f64::powf(tone_mapped_value, 1.0 / 2.2);
            let scaled_value = gamma_corrected_value * 255.0;
            let byte_value = (scaled_value + 0.5) as u8;
            byte_value.to_be_bytes()
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (y * self.width + x) as usize;
                let pixel = self.pixels[i];
                let rgb = pixel.to_rgb();
                writer.write(&correct(rgb.r)).map_err(m)?;
                writer.write(&correct(rgb.g)).map_err(m)?;
                writer.write(&correct(rgb.b)).map_err(m)?;
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
    pub sample_clamp: Option<f64>,
    pub clamp: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FilterConfig {
    Gaussian(GaussianFilterConfig),
    Box,
}

impl FilterConfig {
    pub fn configure(&self) -> Box<dyn Filter> {
        match self {
            FilterConfig::Gaussian(config) => Box::new(GaussianFilter::configure(config)),
            FilterConfig::Box => Box::new(BoxFilter::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GaussianFilterConfig {
    radius: Vector2Config,
    sigma: f64,
}

pub trait Filter: Sync {
    fn radius(&self) -> Vector2;
    fn evaluate(&self, point: Point2) -> f64;
}

pub struct GaussianFilter {
    sigma: f64,
    radius: Vector2,
    exp_x: f64,
    exp_y: f64,
}

impl GaussianFilter {
    pub fn configure(config: &GaussianFilterConfig) -> GaussianFilter {
        let radius = Vector2::configure(&config.radius);
        let sigma = config.sigma;
        GaussianFilter {
            sigma,
            radius,
            exp_x: util::gaussian(radius.x, sigma),
            exp_y: util::gaussian(radius.y, sigma),
        }
    }
}

impl Filter for GaussianFilter {
    fn radius(&self) -> Vector2 {
        self.radius
    }

    fn evaluate(&self, p: Point2) -> f64 {
        f64::max(0.0, util::gaussian(p.x, self.sigma) - self.exp_x)
            * f64::max(0.0, util::gaussian(p.y, self.sigma) - self.exp_y)
    }
}

pub struct BoxFilter {}

impl BoxFilter {
    pub fn new() -> BoxFilter {
        BoxFilter {}
    }
}

impl Filter for BoxFilter {
    fn radius(&self) -> Vector2 {
        Point2::new(0.0, 0.0)
    }

    fn evaluate(&self, _point: Point2) -> f64 {
        1.0
    }
}
