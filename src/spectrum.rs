use std::ops::{Add, Div, Mul};

use serde::{Deserialize, Serialize};

pub type Spectrum = RgbSpectrum;

const LUMINANCE_WEIGHT: [f64; 3] = [0.212671, 0.715160, 0.072169];

#[derive(Copy, Clone)]
pub struct RgbSpectrum {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RgbSpectrum {
    pub fn black() -> RgbSpectrum {
        Spectrum::fill(0.0)
    }

    pub fn fill(v: f64) -> RgbSpectrum {
        RgbSpectrum { r: v, g: v, b: v }
    }

    pub fn mul(&self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }

    pub fn luminance(&self) -> f64 {
        return LUMINANCE_WEIGHT[0] * self.r
            + LUMINANCE_WEIGHT[1] * self.g
            + LUMINANCE_WEIGHT[2] * self.b;
    }

    pub fn to_rgb(&self) -> &RgbSpectrum {
        self
    }
}

impl Add<RgbSpectrum> for RgbSpectrum {
    type Output = RgbSpectrum;
    fn add(self, rhs: RgbSpectrum) -> Self::Output {
        RgbSpectrum {
            r: self.r + rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f64> for RgbSpectrum {
    type Output = RgbSpectrum;
    fn mul(self, rhs: f64) -> Self::Output {
        RgbSpectrum {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Div<f64> for RgbSpectrum {
    type Output = RgbSpectrum;
    fn div(self, rhs: f64) -> Self::Output {
        RgbSpectrum {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpectrumConfig {
    r: f64,
    g: f64,
    b: f64,
}
