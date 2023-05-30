use ndarray::prelude::*;

// MNA Equation matrices
pub struct MNA {
    pub a: Array2<f64>,
    pub b: Array1<f64>,
    pub h: Array2<f64>,
    pub g: Vec<Box<dyn Fn(&Array1<f64>) -> f64>>,
}

impl MNA {
    pub fn new(num_nodes: usize, num_nonlinear_funcs: usize) -> Self {
        MNA {
            a: Array2::zeros((num_nodes, num_nodes)),
            b: Array1::zeros(num_nodes),
            h: Array2::zeros((num_nodes, num_nonlinear_funcs)),
            g: Vec::new(),
        }
    }

    pub fn get_x(&self) -> Array1<f64> {
        Array1::zeros(self.a.nrows())
    }

    pub fn get_err(&self, x: &Array1<f64>) -> Array1<f64> {
        let g_val = self.g.iter().map(|f| f(x)).collect::<Array1<_>>();

        self.a.dot(&x.t()) + self.h.dot(&g_val.t()) - &self.b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_err() {
        let x = array![1.0, 2.0];
        let mna = MNA {
            a: array![[1.0, 2.0], [3.0, 4.0]],
            b: array![8.0, 12.5],
            h: array![[1.0, 0.0], [0.0, 1.0]],
            g: vec![
                Box::new(|x: &Array1<f64>| x.iter().sum()),
                Box::new(|x: &Array1<f64>| x.iter().sum::<f64>() / x.len() as f64),
            ],
        };

        let err_vec = mna.get_err(&x);

        assert_eq!(err_vec, array![0.0, 0.0]);
    }
}
