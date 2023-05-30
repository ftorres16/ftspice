use ndarray::prelude::*;

#[derive(Debug)]
pub struct StateHistory {
    data: Vec<Record>,
}

#[derive(Debug)]
pub struct Record {
    pub n_iters: u64,
    pub x: Array1<f64>,
    pub t: f64,
}

impl StateHistory {
    pub fn new() -> Self {
        StateHistory { data: vec![] }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, n_iters: u64, x: &Array1<f64>, t: f64) {
        self.data.push(Record {
            n_iters,
            x: x.clone(),
            t,
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Record> {
        self.data.iter()
    }

    pub fn pop(&mut self) -> Option<Record> {
        self.data.pop()
    }

    pub fn plte(&self, n: usize) -> Array1<f64> {
        let c3 = -1.0 / 12.0;
        let h_next = self.data[n + 1].t - self.data[n].t;
        let alpha = 6.0 * c3 * h_next.powi(3);

        alpha * &self.divided_diff(n + 1, n - 2)
    }

    pub fn divided_diff(&self, n_max: usize, n_min: usize) -> Array1<f64> {
        if n_max == n_min {
            self.data[n_max].x.clone()
        } else {
            let alpha = 1.0 / (self.data[n_max].t - self.data[n_min].t);

            alpha * (&self.divided_diff(n_max, n_min + 1) - &self.divided_diff(n_max - 1, n_min))
        }
    }
}
