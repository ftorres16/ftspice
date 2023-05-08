use crate::device::stamp;
use crate::device::stamp::Stamp;
use crate::node;

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
        let vc_idx = nodes.get_idx(&self.nodes[0]);
        let vb_idx = nodes.get_idx(&self.nodes[1]);
        let ve_idx = nodes.get_idx(&self.nodes[2]);

        if let Some(i) = vc_idx {
            h_mat[i][g_vec.len()] = 1.0;
        }
        if let Some(i) = vb_idx {
            h_mat[i][g_vec.len() + 1] = 1.0;
        }
        if let Some(i) = ve_idx {
            h_mat[i][g_vec.len() + 2] = 1.0;
        }

        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let vc = match vc_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let vb = match vb_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let ve = match ve_idx {
                Some(i) => x[i],
                None => 0.0,
            };

            let q = model::Model {
                vc: vc,
                vb: vb,
                ve: ve,
            };
            q.ic()
        }));
        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let vc = match vc_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let vb = match vb_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let ve = match ve_idx {
                Some(i) => x[i],
                None => 0.0,
            };

            let q = model::Model {
                vc: vc,
                vb: vb,
                ve: ve,
            };
            q.ib()
        }));
        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let vc = match vc_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let vb = match vb_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let ve = match ve_idx {
                Some(i) => x[i],
                None => 0.0,
            };

            let q = model::Model {
                vc: vc,
                vb: vb,
                ve: ve,
            };
            q.ie()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &node::NodeCollection,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vc_idx = nodes.get_idx(&self.nodes[0]);
        let vb_idx = nodes.get_idx(&self.nodes[1]);
        let ve_idx = nodes.get_idx(&self.nodes[2]);

        let vc = match vc_idx {
            Some(i) => x[i],
            None => 0.0,
        };
        let vb = match vb_idx {
            Some(i) => x[i],
            None => 0.0,
        };
        let ve = match ve_idx {
            Some(i) => x[i],
            None => 0.0,
        };

        let q = model::Model {
            vc: vc,
            vb: vb,
            ve: ve,
        };

        let gee = q.gee();
        let gec = q.gec();
        let gce = q.gce();
        let gcc = q.gcc();
        let i_e = q.ie_eq();
        let i_c = q.ic_eq();

        if let Some(i) = vc_idx {
            a[i][i] += gcc;
            b[i] -= i_c;
        }
        if let Some(i) = vb_idx {
            a[i][i] += gcc + gee - gce - gec;
            b[i] += i_e + i_c;
        }
        if let Some(i) = ve_idx {
            a[i][i] += gee;
            b[i] -= i_e;
        }
        if let (Some(i), Some(j)) = (ve_idx, vc_idx) {
            a[i][j] -= gec;
            a[j][i] -= gce;
        }
        if let (Some(i), Some(j)) = (ve_idx, vb_idx) {
            a[i][j] += gec - gee;
            a[j][i] += gce - gee;
        }
        if let (Some(i), Some(j)) = (vc_idx, vb_idx) {
            a[i][j] += gce - gcc;
            a[j][i] += gec - gcc;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_npn(q: &NPN) -> node::NodeCollection {
        node::parse_elems(&vec![Box::new(q.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let q = NPN {
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        q.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let q = NPN {
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
        };
        let nodes = parse_npn(&q);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

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
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        q.nonlinear_funcs(&nodes, &mut h, &mut g);

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

        let mut x: Vec<f64> = vec![0.0; 3];
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        q.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[n1][n1] > 0.0);
        assert!(a[n2][n2] > 0.0);
        assert!(a[n3][n3] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] > 0.0);
        assert!(b[n3] < 0.0);
    }
}
