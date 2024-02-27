use std::ops::{Add, Div, Mul};

use serde::{Deserialize, Serialize};

pub type Spectrum = RgbSpectrum;

const LUMINANCE_WEIGHT: RgbSpectrum = RgbSpectrum {
    r: 0.212671,
    g: 0.715160,
    b: 0.072169,
};

#[derive(Copy, Clone, Debug)]
pub struct RgbSpectrum {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RgbSpectrum {
    pub fn configure(config: &SpectrumConfig) -> RgbSpectrum {
        RgbSpectrum {
            r: config.r,
            g: config.g,
            b: config.b,
        }
    }

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

    pub fn dot(&self, rhs: RgbSpectrum) -> f64 {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }

    pub fn luminance(&self) -> f64 {
        self.dot(LUMINANCE_WEIGHT)
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
            g: self.g + rhs.g,
            b: self.b + rhs.b,
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

impl PartialEq for RgbSpectrum {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpectrumConfig {
    r: f64,
    g: f64,
    b: f64,
}
