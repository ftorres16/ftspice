use crate::device::stamp;
use crate::device::stamp::Stamp;
use crate::node;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Idd {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
}

impl Stamp for Idd {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> stamp::GType {
        stamp::GType::G1
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn linear_stamp(
        &self,
        nodes: &HashMap<String, node::MNANode>,
        _a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vneg_node = nodes.get(&self.nodes[0]).map(|x| x.idx);
        let vpos_node = nodes.get(&self.nodes[1]).map(|x| x.idx);
        let val = self.val;

        if let Some(i) = vpos_node {
            b[i] += val;
        }
        if let Some(i) = vneg_node {
            b[i] -= val;
        }
    }

    fn undo_linear_stamp(
        &self,
        nodes: &HashMap<String, node::MNANode>,
        _a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vneg_node = nodes.get(&self.nodes[0]).map(|x| x.idx);
        let vpos_node = nodes.get(&self.nodes[1]).map(|x| x.idx);
        let val = self.val;

        if let Some(i) = vpos_node {
            b[i] -= val;
        }
        if let Some(i) = vneg_node {
            b[i] += val;
        }
    }

    fn count_nonlinear_funcs(&self) -> usize {
        0
    }

    fn nonlinear_funcs(
        &self,
        _nodes: &HashMap<String, node::MNANode>,
        _h_mat: &mut Vec<Vec<f64>>,
        _g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
    }

    fn nonlinear_stamp(
        &self,
        _nodes: &HashMap<String, node::MNANode>,
        _x: &Vec<f64>,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_idd(idd: &Idd) -> HashMap<String, node::MNANode> {
        node::parse_elems(&vec![Box::new(idd.clone())])
    }

    #[test]
    fn test_load_idd_node_0_gnd() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("0")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        idd.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0]]);
        assert_eq!(b, [-1e-3]);
    }

    #[test]
    fn test_load_idd_node_1_gnd() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("0"), String::from("1")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        idd.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0]]);
        assert_eq!(b, [1e-3]);
    }

    #[test]
    fn test_load_idd_to_nodes() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        idd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;

        let mut b_model = vec![0.0; nodes.len()];
        b_model[n1] = -1e-3;
        b_model[n2] = 1e-3;

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        idd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(idd.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        idd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, [[0.0], [0.0]]);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_nonlinear_stamp() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        idd.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }
}
