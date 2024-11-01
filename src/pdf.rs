use rand::{distributions::Distribution, Rng};

#[derive(Debug)]
pub struct Pdf {
    pdf: Vec<f64>,
    cdf: Vec<f64>,
}

impl Pdf {
    pub fn new(h: &Vec<f64>) -> Pdf {
        let mut pdf = vec![0.0; h.len()];
        let mut cdf = vec![0.0; h.len()];
        cdf[0] = h[0];
        for k in 1..h.len() {
            cdf[k] = cdf[k - 1] + h[k];
        }
        for k in 0..h.len() {
            pdf[k] = h[k] / cdf[cdf.len() - 1];
            cdf[k] = cdf[k] / cdf[cdf.len() - 1];
        }
        Pdf { pdf, cdf }
    }

    pub fn value(&self, i: usize) -> f64 {
        self.pdf[i]
    }
}

impl Distribution<usize> for Pdf {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let r = rng.gen_range(0.0..1.0);
        for k in 0..self.cdf.len() {
            if r <= self.cdf[k] {
                return k;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::Pdf;

    #[test]
    fn test_pdf_value() {
        let h = vec![10.0, 20.0, 50.0, 15.0, 5.0];
        let pdf = Pdf::new(&h);
        assert_eq!(pdf.value(0), 0.1);
        assert_eq!(pdf.value(1), 0.2);
        assert_eq!(pdf.value(2), 0.5);
        assert_eq!(pdf.value(3), 0.15);
        assert_eq!(pdf.value(4), 0.05);
    }
}
