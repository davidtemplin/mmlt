use std::{
    fs::File,
    io::{self, LineWriter, Write},
};

use crate::spectrum::Spectrum;

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

    pub fn contribute(&mut self, spectrum: Spectrum, x: usize, y: usize) {
        let i = y * self.height + x;
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
