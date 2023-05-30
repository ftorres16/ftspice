use ndarray::prelude::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct SimResult {
    data: Vec<Vec<f64>>,
    headers: Vec<String>,
}

impl SimResult {
    pub fn new(headers: &[&str]) -> Self {
        SimResult {
            data: vec![],
            headers: headers.iter().map(|x| x.to_string()).collect(),
        }
    }

    pub fn push(&mut self, record: HashMap<String, f64>) {
        let row = self.headers.iter().map(|x| record[x]).collect::<Vec<_>>();
        self.data.push(row);
    }

    pub fn print(&self) {
        println!("{}", self.headers.join(","));

        for row in self.data.iter() {
            println!(
                "{}",
                row.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
    }

    pub fn get(&self, label: &str) -> Array1<f64> {
        let idx = self
            .headers
            .iter()
            .position(|x| x == label)
            .expect("Label not found");
        self.data.iter().map(|x| x[idx]).collect()
    }
}
