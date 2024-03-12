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
    pub fn configure(config: &RgbSpectrumConfig) -> RgbSpectrum {
        RgbSpectrum {
            r: config.r,
            g: config.g,
            b: config.b,
        }
    }

    pub fn black() -> RgbSpectrum {
        Spectrum::fill(0.0)
    }

    pub fn is_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
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
        self.r * LUMINANCE_WEIGHT.r + self.g * LUMINANCE_WEIGHT.g + self.b * LUMINANCE_WEIGHT.b
    }

    pub fn to_rgb(&self) -> RgbSpectrum {
        self.clone()
    }

    pub fn has_nans(&self) -> bool {
        self.r.is_nan() || self.g.is_nan() || self.b.is_nan()
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

impl Mul<RgbSpectrum> for f64 {
    type Output = RgbSpectrum;
    fn mul(self, rhs: RgbSpectrum) -> Self::Output {
        RgbSpectrum {
            r: self * rhs.r,
            g: self * rhs.g,
            b: self * rhs.b,
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

pub type SpectrumConfig = RgbSpectrumConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct RgbSpectrumConfig {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[cfg(test)]
mod tests {
    use crate::spectrum::{Spectrum, LUMINANCE_WEIGHT};

    use super::{RgbSpectrum, RgbSpectrumConfig};

    #[test]
    fn test_rgb_spectrum_configure() {
        let config = RgbSpectrumConfig {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        let spectrum = RgbSpectrum::configure(&config);
        assert_eq!(spectrum.r, 1.0);
        assert_eq!(spectrum.g, 1.0);
        assert_eq!(spectrum.b, 1.0);
    }

    #[test]
    fn test_rgb_spectrum_black() {
        let spectrum = RgbSpectrum::black();
        assert_eq!(spectrum.r, 0.0);
        assert_eq!(spectrum.g, 0.0);
        assert_eq!(spectrum.b, 0.0);
    }

    #[test]
    fn test_rgb_spectrum_is_black() {
        let spectrum = RgbSpectrum::black();
        assert!(spectrum.is_black());
    }

    #[test]
    fn test_rgb_spectrum_fill() {
        let spectrum = RgbSpectrum::fill(1.0);
        assert_eq!(spectrum.r, 1.0);
        assert_eq!(spectrum.g, 1.0);
        assert_eq!(spectrum.b, 1.0);
    }

    #[test]
    fn test_rgb_spectrum_mul() {
        let s1 = RgbSpectrum::fill(2.0);
        let s2 = RgbSpectrum::fill(3.0);
        let s3 = s1.mul(s2);
        assert_eq!(s3, RgbSpectrum::fill(6.0));
    }

    #[test]
    fn test_rgb_spectrum_luminance() {
        let spectrum = RgbSpectrum::fill(2.0);
        let luminance = spectrum.luminance();
        let expected_luminance =
            2.0 * LUMINANCE_WEIGHT.r + 2.0 * LUMINANCE_WEIGHT.g + 2.0 * LUMINANCE_WEIGHT.b;
        assert_eq!(luminance, expected_luminance);
    }

    #[test]
    fn test_rgb_spectrum_to_rgb() {
        let spectrum = RgbSpectrum::fill(2.0);
        assert_eq!(spectrum, spectrum.to_rgb());
    }

    #[test]
    fn test_rgb_spectrum_add() {
        let s1 = RgbSpectrum::fill(1.0);
        let s2 = RgbSpectrum::fill(2.0);
        assert_eq!(s1 + s2, RgbSpectrum::fill(3.0));
    }

    #[test]
    fn test_rgb_spectrum_mul_op() {
        let s1 = RgbSpectrum::fill(1.0);
        let s2 = 2.0 * s1;
        assert_eq!(s2, RgbSpectrum::fill(2.0));
        let s3 = s1 * 2.0;
        assert_eq!(s3, RgbSpectrum::fill(2.0));
    }

    #[test]
    fn test_rgb_spectrum_div() {
        let spectrum = RgbSpectrum::fill(2.0);
        assert_eq!(spectrum / 2.0, Spectrum::fill(1.0));
    }

    #[test]
    fn test_rgb_spectrum_eq() {
        let s1 = RgbSpectrum::fill(1.0);
        let s2 = RgbSpectrum::fill(1.0);
        assert_eq!(s1, s2);
    }
}
