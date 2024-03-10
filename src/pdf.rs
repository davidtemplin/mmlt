use rand::{distributions::Distribution, Rng};

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
            if self.cdf[k] <= r {
                return k;
            }
        }
        0
    }
}
