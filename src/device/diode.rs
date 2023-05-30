use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;

mod model;

#[derive(Debug, Clone)]
pub struct Diode {
    pub name: String,
    pub nodes: Vec<String>,
}

impl Stamp for Diode {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> GType {
        GType::G1
    }

    fn get_value(&self) -> f64 {
        unimplemented!()
    }

    fn set_value(&mut self, _value: f64) {
        unimplemented!()
    }

    fn count_nonlinear_funcs(&self) -> usize {
        1
    }

    fn nonlinear_funcs(
        &self,
        nodes: &NodeCollection,
        h_mat: &mut Array2<f64>,
        g_vec: &mut Vec<Box<dyn Fn(&Array1<f64>) -> f64>>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        if let Some(i) = vpos_idx {
            h_mat[(i, g_vec.len())] = 1.0;
        }
        if let Some(i) = vneg_idx {
            h_mat[(i, g_vec.len())] = -1.0;
        }

        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let d = model::Model {
                vpos: vpos_idx.map_or(0.0, |i| x[i]),
                vneg: vneg_idx.map_or(0.0, |i| x[i]),
            };
            d.i()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Array1<f64>,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        let d = model::Model {
            vpos: vpos_idx.map_or(0.0, |i| x[i]),
            vneg: vneg_idx.map_or(0.0, |i| x[i]),
        };
        let g_eq = d.g_eq();
        let i_eq = d.i_eq();

        if let Some(i) = vpos_idx {
            a[(i, i)] += g_eq;
            b[i] -= i_eq;
        }
        if let Some(i) = vneg_idx {
            a[(i, i)] += g_eq;
            b[i] += i_eq;
        }
        if let (Some(i), Some(j)) = (vpos_idx, vneg_idx) {
            a[(i, j)] -= g_eq;
            a[(j, i)] -= g_eq;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_dio(dio: &Diode) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(dio.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let dio = Diode {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        dio.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let dio = Diode {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        dio.linear_stamp(&nodes, &mut a, &mut b);
        dio.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(dio.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_node_0_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("0"), String::from("1")],
        };
        let nodes = parse_dio(&dio);
        let mut h = Array2::zeros((1, 1));
        let mut g = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, array![[-1.0]]);
        assert_eq!(g.len(), 1);

        let x_test = array![1.5, 1.0];
        assert!(g[0](&x_test) < 0.0);
    }

    #[test]
    fn test_nonlinear_funcs_node_1_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("0")],
        };
        let nodes = parse_dio(&dio);
        let mut h = Array2::zeros((1, 1));
        let mut g = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, array![[1.0]]);
        assert_eq!(g.len(), 1);

        let x_test = array![1.5, 1.0];
        assert!(g[0](&x_test) > 0.0);
    }

    #[test]
    fn test_nonlinear_funcs_diode_two_nodes() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        let mut h_model = Array2::zeros((2, 1));
        h_model[(n1, 0)] = 1.0;
        h_model[(n2, 0)] = -1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 1);

        let mut x_test = Array1::zeros(2);
        x_test[n1] = 1.5;
        x_test[n2] = 1.0;

        assert!(g[0](&x_test) > 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_node_0_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("0"), String::from("1")],
        };
        let nodes = parse_dio(&dio);
        let x = array![1.0];
        let mut a = Array2::zeros((1, 1));
        let mut b = Array1::zeros(1);

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[(0, 0)] > 0.0);
        assert!(b[0] < 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_node_1_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("0")],
        };
        let nodes = parse_dio(&dio);
        let x = array![1.0];
        let mut a = Array2::zeros((1, 1));
        let mut b = Array1::zeros(1);

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[(0, 0)] > 0.0);
        assert!(b[0] > 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_two_nodes() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let x = array![1.0, 2.0];
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        assert!(a[(n1, n1)] > 0.0);
        assert!(a[(n1, n2)] < 0.0);
        assert!(a[(n2, n1)] < 0.0);
        assert!(a[(n2, n2)] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] < 0.0);
    }
}
