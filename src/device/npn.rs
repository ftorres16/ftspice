use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;

mod model;

#[derive(Debug, Clone)]
pub struct NPN {
    pub name: String,
    pub nodes: Vec<String>,
}

impl Stamp for NPN {
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
        3
    }

    fn nonlinear_funcs(
        &self,
        nodes: &NodeCollection,
        h_mat: &mut Array2<f64>,
        g_vec: &mut Vec<Box<dyn Fn(&Array1<f64>) -> f64>>,
    ) {
        let vc_idx = nodes.get_idx(&self.nodes[0]);
        let vb_idx = nodes.get_idx(&self.nodes[1]);
        let ve_idx = nodes.get_idx(&self.nodes[2]);

        if let Some(i) = vc_idx {
            h_mat[(i, g_vec.len())] = 1.0;
        }
        if let Some(i) = vb_idx {
            h_mat[(i, g_vec.len() + 1)] = 1.0;
        }
        if let Some(i) = ve_idx {
            h_mat[(i, g_vec.len() + 2)] = 1.0;
        }

        fn get_model(
            vc_idx: Option<usize>,
            vb_idx: Option<usize>,
            ve_idx: Option<usize>,
            x: &Array1<f64>,
        ) -> model::Model {
            model::Model {
                vc: vc_idx.map_or(0.0, |i| x[i]),
                vb: vb_idx.map_or(0.0, |i| x[i]),
                ve: ve_idx.map_or(0.0, |i| x[i]),
            }
        }

        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let q = get_model(vc_idx, vb_idx, ve_idx, x);
            q.ic()
        }));
        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let q = get_model(vc_idx, vb_idx, ve_idx, x);
            q.ib()
        }));
        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let q = get_model(vc_idx, vb_idx, ve_idx, x);
            q.ie()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Array1<f64>,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        let vc_idx = nodes.get_idx(&self.nodes[0]);
        let vb_idx = nodes.get_idx(&self.nodes[1]);
        let ve_idx = nodes.get_idx(&self.nodes[2]);

        let vc = vc_idx.map_or(0.0, |i| x[i]);
        let vb = vb_idx.map_or(0.0, |i| x[i]);
        let ve = ve_idx.map_or(0.0, |i| x[i]);

        let q = model::Model { vc, vb, ve };

        let gee = q.gee();
        let gec = q.gec();
        let gce = q.gce();
        let gcc = q.gcc();
        let i_e = q.ie_eq();
        let i_c = q.ic_eq();

        if let Some(i) = vc_idx {
            a[(i, i)] += gcc;
            b[i] -= i_c;
        }
        if let Some(i) = vb_idx {
            a[(i, i)] += gcc + gee - gce - gec;
            b[i] += i_e + i_c;
        }
        if let Some(i) = ve_idx {
            a[(i, i)] += gee;
            b[i] -= i_e;
        }
        if let (Some(i), Some(j)) = (ve_idx, vc_idx) {
            a[(i, j)] -= gec;
            a[(j, i)] -= gce;
        }
        if let (Some(i), Some(j)) = (ve_idx, vb_idx) {
            a[(i, j)] += gec - gee;
            a[(j, i)] += gce - gee;
        }
        if let (Some(i), Some(j)) = (vc_idx, vb_idx) {
            a[(i, j)] += gce - gcc;
            a[(j, i)] += gec - gcc;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_npn(q: &NPN) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(q.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let q = NPN {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        q.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let q = NPN {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        q.linear_stamp(&nodes, &mut a, &mut b);
        q.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let q = NPN {
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut h = Array2::zeros((3, 3));
        let mut g = Vec::new();

        q.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(q.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs() {
        let q = NPN {
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut h = Array2::zeros((3, 3));
        let mut g = Vec::new();

        q.nonlinear_funcs(&nodes, &mut h, &mut g);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let n3 = nodes.get_idx("3").unwrap();

        let mut h_model = Array2::zeros(h.raw_dim());
        h_model[(n1, 0)] = 1.0;
        h_model[(n2, 1)] = 1.0;
        h_model[(n3, 2)] = 1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 3);

        let mut x_test = Array1::zeros(3);
        x_test[n1] = 2.0;
        x_test[n2] = 1.0;
        x_test[n3] = 0.0;

        assert!(g[0](&x_test) > 0.0);
        assert!(g[1](&x_test) > 0.0);
        assert!(g[2](&x_test) < 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_three_nodes() {
        let q = NPN {
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let n3 = nodes.get_idx("3").unwrap();

        let mut x = Array1::zeros(3);
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a = Array2::zeros((3, 3));
        let mut b = Array1::zeros(3);

        q.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[(n1, n1)] > 0.0);
        assert!(a[(n2, n2)] > 0.0);
        assert!(a[(n3, n3)] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] > 0.0);
        assert!(b[n3] < 0.0);
    }
}
