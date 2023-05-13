use crate::engine::linalg::{mat_vec_prod, vec_add, vec_sub};

// MNA Equation matrices
pub struct MNA {
    pub a: Vec<Vec<f64>>,
    pub b: Vec<f64>,
    pub h: Vec<Vec<f64>>,
    pub g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
}

impl MNA {
    pub fn new(num_nodes: usize, num_nonlinear_funcs: usize) -> Self {
        MNA {
            a: vec![vec![0.0; num_nodes]; num_nodes],
            b: vec![0.0; num_nodes],
            h: vec![vec![0.0; num_nonlinear_funcs]; num_nodes],
            g: Vec::new(),
        }
    }

    pub fn get_x(&self) -> Vec<f64> {
        vec![0.0; self.a.len()]
    }

    pub fn get_err(&self, x: &Vec<f64>) -> Vec<f64> {
        let g_val = self.g.iter().map(|f| f(x)).collect::<Vec<_>>();
        let h_times_g = mat_vec_prod(&self.h, &g_val);
        let a_times_x = mat_vec_prod(&self.a, x);
        let f = vec_add(&a_times_x, &h_times_g);

        vec_sub(&f, &self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_err() {
        let x: Vec<f64> = vec![1.0, 2.0];
        let mna = MNA {
            a: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            b: vec![8.0, 12.5],
            h: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            g: vec![
                Box::new(|x: &Vec<f64>| x.iter().sum()),
                Box::new(|x: &Vec<f64>| x.iter().sum::<f64>() / x.len() as f64),
            ],
        };

        let err_vec = mna.get_err(&x);

        assert_eq!(err_vec, [0.0, 0.0]);
    }
}
