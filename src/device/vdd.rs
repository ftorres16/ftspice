use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;
use crate::spice_fn::SpiceFn;

#[derive(Debug, Clone)]
pub struct Vdd {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
    pub tran_fn: Option<SpiceFn>,
}

impl Stamp for Vdd {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> GType {
        GType::G2
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

    fn linear_stamp(&self, nodes: &NodeCollection, a: &mut Array2<f64>, b: &mut Array1<f64>) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for source.");

        b[is_idx] = self.val;

        if let Some(i) = vpos_idx {
            a[(is_idx, i)] += 1.0;
            a[(i, is_idx)] += 1.0;
        }

        if let Some(i) = vneg_idx {
            a[(is_idx, i)] -= 1.0;
            a[(i, is_idx)] -= 1.0;
        }
    }

    fn undo_linear_stamp(&self, nodes: &NodeCollection, a: &mut Array2<f64>, b: &mut Array1<f64>) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for source.");

        b[is_idx] = 0.0;

        if let Some(i) = vpos_idx {
            a[(is_idx, i)] -= 1.0;
            a[(i, is_idx)] -= 1.0;
        }

        if let Some(i) = vneg_idx {
            a[(is_idx, i)] += 1.0;
            a[(i, is_idx)] += 1.0;
        }
    }

    fn count_nonlinear_funcs(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_vdd(vdd: &Vdd) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(vdd.clone())])
    }

    fn test_vdd(nodes: &[&str]) -> Vdd {
        Vdd {
            name: String::from("V1"),
            nodes: nodes.iter().map(|s| s.to_string()).collect(),
            val: 1e-3,
            tran_fn: None,
        }
    }

    #[test]
    fn test_linear_stamp_vdd_node_0_gnd() {
        let vdd = test_vdd(&["0", "1"]);
        let nodes = parse_vdd(&vdd);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = Array2::zeros((nodes.len(), nodes.len()));
        let mut b_model = Array1::zeros(nodes.len());
        a_model[(n1, v1)] = 1.0;
        a_model[(v1, n1)] = 1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_linear_stamp_vdd_node_1_gnd() {
        let vdd = test_vdd(&["1", "0"]);
        let nodes = parse_vdd(&vdd);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = Array2::zeros((nodes.len(), nodes.len()));
        let mut b_model = Array1::zeros(nodes.len());
        a_model[(n1, v1)] = -1.0;
        a_model[(v1, n1)] = -1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_linear_stamp_vdd_to_nodes() {
        let vdd = test_vdd(&["1", "2"]);
        let nodes = parse_vdd(&vdd);
        let mut a = Array2::zeros((3, 3));
        let mut b = Array1::zeros(3);

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = Array2::zeros((nodes.len(), nodes.len()));
        let mut b_model = Array1::zeros(nodes.len());
        a_model[(n1, v1)] = -1.0;
        a_model[(v1, n1)] = -1.0;
        a_model[(n2, v1)] = 1.0;
        a_model[(v1, n2)] = 1.0;
        b_model[n1] = 0.0;
        b_model[n2] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let vdd = test_vdd(&["1", "2"]);
        let nodes = parse_vdd(&vdd);
        let mut a = Array2::zeros((3, 3));
        let mut b = Array1::zeros(3);

        vdd.linear_stamp(&nodes, &mut a, &mut b);
        vdd.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, Array2::zeros((3, 3)));
        assert_eq!(b, Array1::zeros(3));
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let vdd = test_vdd(&["1", "2"]);
        let nodes = parse_vdd(&vdd);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        vdd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(vdd.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_() {
        let vdd = test_vdd(&["1", "2"]);
        let nodes = parse_vdd(&vdd);
        let mut h = Array2::zeros((2, 1));
        let mut g = Vec::new();

        vdd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, array![[0.0], [0.0]]);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_nonlinear_stamp() {
        let vdd = test_vdd(&["1", "2"]);
        let nodes = parse_vdd(&vdd);
        let x = array![1.0, 2.0];
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        vdd.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }
}
