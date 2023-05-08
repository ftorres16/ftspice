use crate::device::stamp;
use crate::device::stamp::Stamp;
use crate::node;

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

    fn gtype(&self) -> stamp::GType {
        stamp::GType::G1
    }

    fn set_value(&mut self, _value: f64) {
        unimplemented!()
    }

    fn linear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn undo_linear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn count_nonlinear_funcs(&self) -> usize {
        3
    }

    fn nonlinear_funcs(
        &self,
        nodes: &node::NodeCollection,
        h_mat: &mut Vec<Vec<f64>>,
        g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
        let vd_idx = nodes.get_idx(&self.nodes[0]);
        let vg_idx = nodes.get_idx(&self.nodes[1]);
        let vs_idx = nodes.get_idx(&self.nodes[2]);

        if let Some(i) = vd_idx {
            h_mat[i][g_vec.len()] = 1.0;
        }
        if let Some(i) = vg_idx {
            h_mat[i][g_vec.len() + 1] = 1.0;
        }
        if let Some(i) = vs_idx {
            h_mat[i][g_vec.len() + 2] = 1.0;
        }

        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let mut vd = vd_idx.map_or(0.0, |i| x[i]);
            let vg = vg_idx.map_or(0.0, |i| x[i]);
            let mut vs = vs_idx.map_or(0.0, |i| x[i]);

            if vs > vd {
                (vs, vd) = (vd, vs);
            }

            let m = model::Model {
                vd: vd,
                vg: vg,
                vs: vs,
            };
            m.id()
        }));
        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let mut vd = vd_idx.map_or(0.0, |i| x[i]);
            let vg = vg_idx.map_or(0.0, |i| x[i]);
            let mut vs = vs_idx.map_or(0.0, |i| x[i]);

            if vs > vd {
                (vs, vd) = (vd, vs);
            }

            let m = model::Model {
                vd: vd,
                vg: vg,
                vs: vs,
            };
            m.ig()
        }));
        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let mut vd = vd_idx.map_or(0.0, |i| x[i]);
            let vg = vg_idx.map_or(0.0, |i| x[i]);
            let mut vs = vs_idx.map_or(0.0, |i| x[i]);

            if vs > vd {
                (vs, vd) = (vd, vs);
            }

            let m = model::Model {
                vd: vd,
                vg: vg,
                vs: vs,
            };
            m.is()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &node::NodeCollection,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
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

        let m = model::Model {
            vd: vd,
            vg: vg,
            vs: vs,
        };

        let gds = m.gds();
        let gm = m.gm();
        let ieq = m.ieq();

        if let Some(i) = vd_idx {
            a[i][i] += gds;
            b[i] -= ieq;
        }
        if let Some(i) = vs_idx {
            a[i][i] += gds + gm;
            b[i] += ieq;
        }
        if let (Some(i), Some(j)) = (vd_idx, vs_idx) {
            a[i][j] -= gds + gm;
            a[j][i] -= gds;
        }
        if let (Some(i), Some(j)) = (vd_idx, vg_idx) {
            a[i][j] += gm;
        }
        if let (Some(i), Some(j)) = (vs_idx, vg_idx) {
            a[i][j] -= gm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_nmos(m: &NMOS) -> node::NodeCollection {
        node::parse_elems(&vec![Box::new(m.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        m.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let m = NMOS {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_nmos(&m);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

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
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        m.nonlinear_funcs(&nodes, &mut h, &mut g);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();
        let n3 = nodes.get_idx("3").unwrap();

        let mut h_model = vec![vec![0.0; h[0].len()]; h.len()];
        h_model[n1][0] = 1.0;
        h_model[n2][1] = 1.0;
        h_model[n3][2] = 1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 3);

        let mut x_test: Vec<f64> = vec![0.0; 3];
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

        let mut x: Vec<f64> = vec![0.0; 3];
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        m.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[n1][n1] > 0.0);
        assert!(a[n1][n2] > 0.0);
        assert!(a[n1][n3] < 0.0);
        assert_eq!(a[n2], [0.0, 0.0, 0.0]);
        assert!(a[n3][n1] < 0.0);
        assert!(a[n3][n2] < 0.0);
        assert!(a[n3][n3] > 0.0);

        assert!(b[n1] > 0.0);
        assert_eq!(b[n2], 0.0);
        assert!(b[n3] < 0.0);
    }
}
