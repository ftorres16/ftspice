use crate::device::stamp;
use crate::device::stamp::Stamp;
use crate::node;

#[derive(Debug, Clone)]
pub struct Vdd {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
}

impl Stamp for Vdd {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> stamp::GType {
        stamp::GType::G2
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn linear_stamp(&self, nodes: &node::NodeCollection, a: &mut Vec<Vec<f64>>, b: &mut Vec<f64>) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for source.");

        b[is_idx] += self.val;

        if let Some(i) = vpos_idx {
            a[is_idx][i] += 1.0;
            a[i][is_idx] += 1.0;
        }

        if let Some(i) = vneg_idx {
            a[is_idx][i] -= 1.0;
            a[i][is_idx] -= 1.0;
        }
    }

    fn undo_linear_stamp(
        &self,
        nodes: &node::NodeCollection,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for source.");

        b[is_idx] -= self.val;

        if let Some(i) = vpos_idx {
            a[is_idx][i] -= 1.0;
            a[i][is_idx] -= 1.0;
        }

        if let Some(i) = vneg_idx {
            a[is_idx][i] += 1.0;
            a[i][is_idx] += 1.0;
        }
    }

    fn count_nonlinear_funcs(&self) -> usize {
        0
    }

    fn nonlinear_funcs(
        &self,
        _nodes: &node::NodeCollection,
        _h_mat: &mut Vec<Vec<f64>>,
        _g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
    }

    fn nonlinear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _x: &Vec<f64>,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_vdd(vdd: &Vdd) -> node::NodeCollection {
        node::parse_elems(&vec![Box::new(vdd.clone())])
    }

    #[test]
    fn test_load_vdd_node_0_gnd() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("0"), String::from("1")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = 1.0;
        a_model[v1][n1] = 1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_vdd_node_1_gnd() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("0")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = -1.0;
        a_model[v1][n1] = -1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_vdd_to_nodes() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        vdd.linear_stamp(&nodes, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let v1 = nodes.get_idx("V1").unwrap();

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = -1.0;
        a_model[v1][n1] = -1.0;
        a_model[n2][v1] = 1.0;
        a_model[v1][n2] = 1.0;
        b_model[n1] = 0.0;
        b_model[n2] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        vdd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(vdd.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        vdd.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, [[0.0], [0.0]]);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_nonlinear_stamp() {
        let vdd = Vdd {
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e-3,
        };
        let nodes = parse_vdd(&vdd);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        vdd.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }
}
