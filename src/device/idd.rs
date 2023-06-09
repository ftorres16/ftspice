use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;
use crate::spice_fn::SpiceFn;

#[derive(Debug, Clone)]
pub struct Idd {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
    pub tran_fn: Option<SpiceFn>,
}

impl Stamp for Idd {
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
        self.val
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn has_tran(&self) -> bool {
        self.tran_fn.is_some()
    }

    fn eval_tran(&mut self, t: &f64) {
        if let Some(f) = &self.tran_fn {
            self.val = f.eval(t);
        }
    }

    fn linear_stamp(&self, nodes: &NodeCollection, _a: &mut Array2<f64>, b: &mut Array1<f64>) {
        let vneg_node = nodes.get_idx(&self.nodes[0]);
        let vpos_node = nodes.get_idx(&self.nodes[1]);
        let val = self.val;

        if let Some(i) = vpos_node {
            b[i] += val;
        }
        if let Some(i) = vneg_node {
            b[i] -= val;
        }
    }

    fn undo_linear_stamp(&self, nodes: &NodeCollection, _a: &mut Array2<f64>, b: &mut Array1<f64>) {
        let vneg_node = nodes.get_idx(&self.nodes[0]);
        let vpos_node = nodes.get_idx(&self.nodes[1]);
        let val = self.val;

        if let Some(i) = vpos_node {
            b[i] -= val;
        }
        if let Some(i) = vneg_node {
            b[i] += val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_idd(idd: &Idd) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(idd.clone())])
    }

    fn test_idd(nodes: &[&str]) -> Idd {
        Idd {
            name: String::from("I1"),
            nodes: nodes.iter().map(|s| s.to_string()).collect(),
            val: 1e-3,
            tran_fn: None,
        }
    }

    #[test]
    fn test_linear_stamp_idd_node_0_gnd() {
        let idd = test_idd(&["1", "0"]);
        let nodes = parse_idd(&idd);
        let mut a = Array2::zeros((1, 1));
        let mut b = Array1::zeros(1);

        idd.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0]]);
        assert_eq!(b, array![-1e-3]);
    }

    #[test]
    fn test_linear_stamp_idd_node_1_gnd() {
        let idd = test_idd(&["0", "1"]);
        let nodes = parse_idd(&idd);
        let mut a = Array2::zeros((1, 1));
        let mut b = Array1::zeros(1);

        idd.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0]]);
        assert_eq!(b, array![1e-3]);
    }

    #[test]
    fn test_linear_stamp_idd_to_nodes() {
        let idd = test_idd(&["1", "2"]);
        let nodes = parse_idd(&idd);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        idd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        let mut b_model = Array1::zeros(nodes.len());
        b_model[n1] = -1e-3;
        b_model[n2] = 1e-3;

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let idd = test_idd(&["1", "2"]);
        let nodes = parse_idd(&idd);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        idd.linear_stamp(&nodes, &mut a, &mut b);
        idd.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let idd = test_idd(&["1", "2"]);
        let nodes = parse_idd(&idd);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        idd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(idd.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_() {
        let idd = test_idd(&["1", "2"]);
        let nodes = parse_idd(&idd);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        idd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, array![[0.0], [0.0]]);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_nonlinear_stamp() {
        let idd = test_idd(&["1", "2"]);
        let nodes = parse_idd(&idd);
        let x = array![1.0, 2.0];
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        idd.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }
}
