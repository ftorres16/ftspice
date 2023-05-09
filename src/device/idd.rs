use crate::device::{GType, Stamp};
use crate::node;

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

    fn gtype(&self) -> GType {
        GType::G1
    }

    fn get_value(&self) -> f64 {
        self.val
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn linear_stamp(&self, nodes: &node::NodeCollection, _a: &mut Vec<Vec<f64>>, b: &mut Vec<f64>) {
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

    fn undo_linear_stamp(
        &self,
        nodes: &node::NodeCollection,
        _a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
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

    fn parse_idd(idd: &Idd) -> node::NodeCollection {
        node::parse_elems(&vec![Box::new(idd.clone())])
    }

    #[test]
    fn test_linear_stamp_idd_node_0_gnd() {
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
    fn test_linear_stamp_idd_node_1_gnd() {
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
    fn test_linear_stamp_idd_to_nodes() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        idd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        let mut b_model = vec![0.0; nodes.len()];
        b_model[n1] = -1e-3;
        b_model[n2] = 1e-3;

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let idd = Idd {
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_idd(&idd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        idd.linear_stamp(&nodes, &mut a, &mut b);
        idd.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
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
