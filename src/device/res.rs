use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;

#[derive(Debug, Clone)]
pub struct Res {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
}

impl Stamp for Res {
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

    fn linear_stamp(&self, nodes: &NodeCollection, a: &mut Vec<Vec<f64>>, _b: &mut Vec<f64>) {
        let g = 1.0 / self.val;

        let vneg_node = nodes.get_idx(&self.nodes[0]);
        let vpos_node = nodes.get_idx(&self.nodes[1]);

        if let Some(i) = vneg_node {
            a[i][i] += g;
        }
        if let Some(i) = vpos_node {
            a[i][i] += g;
        }
        if let (Some(i), Some(j)) = (vpos_node, vneg_node) {
            a[i][j] -= g;
            a[j][i] -= g;
        }
    }

    fn undo_linear_stamp(&self, nodes: &NodeCollection, a: &mut Vec<Vec<f64>>, _b: &mut Vec<f64>) {
        let g = 1.0 / self.val;

        let vneg_node = nodes.get_idx(&self.nodes[0]);
        let vpos_node = nodes.get_idx(&self.nodes[1]);

        if let Some(i) = vneg_node {
            a[i][i] -= g;
        }
        if let Some(i) = vpos_node {
            a[i][i] -= g;
        }
        if let (Some(i), Some(j)) = (vpos_node, vneg_node) {
            a[i][j] += g;
            a[j][i] += g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_res(res: &Res) -> node::NodeCollection {
        node::NodeCollection::from_elems(&vec![Box::new(res.clone())])
    }

    #[test]
    fn test_linear_stamp_node_0_gnd() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("0"), String::from("1")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]];
        let mut b: Vec<f64> = vec![0.0; 1];

        res.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[1e-3]]);
        assert_eq!(b, [0.0]);
    }

    #[test]
    fn test_linear_stamp_node_1_gnd() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("0")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]];
        let mut b: Vec<f64> = vec![0.0; 1];

        res.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[1e-3]]);
        assert_eq!(b, [0.0]);
    }

    #[test]
    fn test_linear_stamp_to_nodes() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        res.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        a_model[n1][n1] = 1e-3;
        a_model[n1][n2] = -1e-3;
        a_model[n2][n1] = -1e-3;
        a_model[n2][n2] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, [0.0, 0.0]);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        res.linear_stamp(&nodes, &mut a, &mut b);
        res.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, vec![vec![0.0; 3]; 3]);
        assert_eq!(b, vec![0.0; 3]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        res.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(res.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e3,
        };
        let nodes = parse_res(&res);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        res.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, [[0.0], [0.0]]);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_nonlinear_stamp() {
        let res = Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_res(&res);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        res.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }
}
