use ndarray::prelude::*;

use crate::device::{GType, Stamp};
use crate::node_collection::NodeCollection;

mod model;

#[derive(Debug, Clone)]
pub struct NMOS {
    pub name: String,
    pub nodes: Vec<String>,
}

impl Stamp for NMOS {
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
        let vd_idx = nodes.get_idx(&self.nodes[0]);
        let vg_idx = nodes.get_idx(&self.nodes[1]);
        let vs_idx = nodes.get_idx(&self.nodes[2]);

        if let Some(i) = vd_idx {
            h_mat[(i, g_vec.len())] = 1.0;
        }
        if let Some(i) = vg_idx {
            h_mat[(i, g_vec.len() + 1)] = 1.0;
        }
        if let Some(i) = vs_idx {
            h_mat[(i, g_vec.len() + 2)] = 1.0;
        }

        fn get_model(
            vd_idx: Option<usize>,
            vg_idx: Option<usize>,
            vs_idx: Option<usize>,
            x: &Array1<f64>,
        ) -> model::Model {
            let mut vd = vd_idx.map_or(0.0, |i| x[i]);
            let vg = vg_idx.map_or(0.0, |i| x[i]);
            let mut vs = vs_idx.map_or(0.0, |i| x[i]);

            if vs > vd {
                (vs, vd) = (vd, vs);
            }

            model::Model { vd, vg, vs }
        }

        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let m = get_model(vd_idx, vg_idx, vs_idx, x);
            m.id()
        }));
        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let m = get_model(vd_idx, vg_idx, vs_idx, x);
            m.ig()
        }));
        g_vec.push(Box::new(move |x: &Array1<f64>| {
            let m = get_model(vd_idx, vg_idx, vs_idx, x);
            m.is()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Array1<f64>,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        let mut vd_idx = nodes.get_idx(&self.nodes[0]);
        let vg_idx = nodes.get_idx(&self.nodes[1]);
        let mut vs_idx = nodes.get_idx(&self.nodes[2]);

        let mut vd = vd_idx.map_or(0.0, |i| x[i]);
        let vg = vg_idx.map_or(0.0, |i| x[i]);
        let mut vs = vs_idx.map_or(0.0, |i| x[i]);

        if vs > vd {
            (vd, vs) = (vs, vd);
            (vd_idx, vs_idx) = (vs_idx, vd_idx);
        }

        let m = model::Model { vd, vg, vs };

        let gds = m.gds();
        let gm = m.gm();
        let ieq = m.ieq();

        if let Some(i) = vd_idx {
            a[(i, i)] += gds;
            b[i] -= ieq;
        }
        if let Some(i) = vs_idx {
            a[(i, i)] += gds + gm;
            b[i] += ieq;
        }
        if let (Some(i), Some(j)) = (vd_idx, vs_idx) {
            a[(i, j)] -= gds + gm;
            a[(j, i)] -= gds;
        }
        if let (Some(i), Some(j)) = (vd_idx, vg_idx) {
            a[(i, j)] += gm;
        }
        if let (Some(i), Some(j)) = (vs_idx, vg_idx) {
            a[(i, j)] -= gm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_nmos(m: &NMOS) -> NodeCollection {
        NodeCollection::from_elems(&[Box::new(m.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        m.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_undo_linear_stamp() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut a = Array2::zeros((2, 2));
        let mut b = Array1::zeros(2);

        m.linear_stamp(&nodes, &mut a, &mut b);
        m.undo_linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, array![[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, array![0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut h = Array2::zeros((3, 3));
        let mut g = Vec::new();

        m.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(m.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut h = Array2::zeros((3, 3));
        let mut g = Vec::new();

        m.nonlinear_funcs(&nodes, &mut h, &mut g);

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
        assert_eq!(g[1](&x_test), 0.0);
        assert!(g[2](&x_test) < 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_three_nodes() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let n3 = nodes.get_idx("3").unwrap();

        let mut x = Array1::zeros(3);
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a = Array2::zeros((3, 3));
        let mut b = Array1::zeros(3);

        m.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[(n1, n1)] > 0.0);
        assert!(a[(n1, n2)] > 0.0);
        assert!(a[(n1, n3)] < 0.0);
        assert_eq!(a.slice(s!(n2, ..)), array![0.0, 0.0, 0.0]);
        assert!(a[(n3, n1)] < 0.0);
        assert!(a[(n3, n2)] < 0.0);
        assert!(a[(n3, n3)] > 0.0);

        assert!(b[n1] > 0.0);
        assert_eq!(b[n2], 0.0);
        assert!(b[n3] < 0.0);
    }
}
