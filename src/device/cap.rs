use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;

mod model;

#[derive(Debug, Clone)]
pub struct Cap {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
    pub u_curr: Option<f64>,
    pub i_curr: Option<f64>,
}

impl Stamp for Cap {
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

    fn init_state(&mut self, nodes: &NodeCollection, x: &Array1<f64>) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        self.u_curr = Some(vpos - vneg);
        self.i_curr = Some(0.0);
    }

    fn update_state(&mut self, nodes: &NodeCollection, x: &Array1<f64>, h: &f64) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let c = model::Model {
            vpos: vpos,
            vneg: vneg,
            val: self.val,
            u_old: self.u_curr.expect("Cap voltage history not initialized"),
            i_old: self.i_curr.expect("Cap current history not initialized"),
        };

        self.u_curr = Some(c.u_new());
        self.i_curr = Some(c.i_new(h));
    }

    fn dynamic_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Array1<f64>,
        h: &f64,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let c = model::Model {
            vpos: vpos,
            vneg: vneg,
            val: self.val,
            u_old: self.u_curr.expect("Cap voltage history not initialized"),
            i_old: self.i_curr.expect("Cap current history not initialized"),
        };

        let g_eq = c.g_eq(h);
        let i_eq = c.i_eq(h);

        if let Some(i) = vpos_idx {
            a[(i, i)] += g_eq;
            b[i] -= i_eq;
        }
        if let Some(i) = vneg_idx {
            a[(i, i)] += g_eq;
            b[i] += i_eq;
        }
        if let (Some(i), Some(j)) = (vneg_idx, vpos_idx) {
            a[(i, j)] -= g_eq;
            a[(j, i)] -= g_eq;
        }
    }

    fn undo_dynamic_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Array1<f64>,
        h: &f64,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let c = model::Model {
            vpos: vpos,
            vneg: vneg,
            val: self.val,
            u_old: self.u_curr.expect("Cap voltage history not initialized"),
            i_old: self.i_curr.expect("Cap current history not initialized"),
        };

        let g_eq = c.g_eq(h);
        let i_eq = c.i_eq(h);

        if let Some(i) = vpos_idx {
            a[(i, i)] -= g_eq;
            b[i] += i_eq;
        }
        if let Some(i) = vneg_idx {
            a[(i, i)] -= g_eq;
            b[i] -= i_eq;
        }
        if let (Some(i), Some(j)) = (vneg_idx, vpos_idx) {
            a[(i, j)] += g_eq;
            a[(j, i)] += g_eq;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_cap(cap: &Cap) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(cap.clone())])
    }

    fn test_cap(nodes: &[&str]) -> Cap {
        Cap {
            name: String::from("C1"),
            nodes: nodes.iter().map(|s| s.to_string()).collect(),
            val: 1e-6,
            u_curr: Some(3.0),
            i_curr: Some(0.0),
        }
    }

    #[test]
    fn test_dynamic_stamp() {
        let cap = test_cap(&["1", "2"]);
        let nodes = parse_cap(&cap);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);
        let x = array![1.0, 2.0];
        let h = 1e-8;

        cap.dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        assert!(a[(n1, n1)] > 0.0);
        assert!(a[(n1, n2)] < 0.0);
        assert!(a[(n2, n1)] < 0.0);
        assert!(a[(n2, n2)] > 0.0);

        assert!(b[n1] < 0.0);
        assert!(b[n2] > 0.0);
    }

    #[test]
    fn test_undo_dynamic_stamp() {
        let cap = test_cap(&["1", "2"]);
        let nodes = parse_cap(&cap);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);
        let x = array![1.0, 2.0];
        let h = 1e-8;

        cap.dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);
        cap.undo_dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }
}
