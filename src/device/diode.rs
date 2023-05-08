use crate::device::stamp;
use crate::device::stamp::Stamp;
use crate::node;

mod model;

#[derive(Debug, Clone)]
pub struct Diode {
    pub name: String,
    pub nodes: Vec<String>,
}

impl Stamp for Diode {
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
        1
    }

    fn nonlinear_funcs(
        &self,
        nodes: &node::NodeCollection,
        h_mat: &mut Vec<Vec<f64>>,
        g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        if let Some(i) = vpos_idx {
            h_mat[i][g_vec.len()] = 1.0;
        }
        if let Some(i) = vneg_idx {
            h_mat[i][g_vec.len()] = -1.0;
        }

        g_vec.push(Box::new(move |x: &Vec<f64>| {
            let vpos = match vpos_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let vneg = match vneg_idx {
                Some(i) => x[i],
                None => 0.0,
            };
            let d = model::Model {
                vpos: vpos,
                vneg: vneg,
            };
            d.i()
        }));
    }

    fn nonlinear_stamp(
        &self,
        nodes: &node::NodeCollection,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = match vpos_idx {
            Some(i) => x[i],
            None => 0.0,
        };
        let vneg = match vneg_idx {
            Some(i) => x[i],
            None => 0.0,
        };

        let d = model::Model {
            vpos: vpos,
            vneg: vneg,
        };
        let g_eq = d.g_eq();
        let i_eq = d.i_eq();

        if let Some(i) = vpos_idx {
            a[i][i] += g_eq;
            b[i] -= i_eq;
        }
        if let Some(i) = vneg_idx {
            a[i][i] += g_eq;
            b[i] += i_eq;
        }
        if let (Some(i), Some(j)) = (vpos_idx, vneg_idx) {
            a[i][j] -= g_eq;
            a[j][i] -= g_eq;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_dio(dio: &Diode) -> node::NodeCollection {
        node::parse_elems(&vec![Box::new(dio.clone())])
    }

    #[test]
    fn test_linear_stamp() {
        let dio = Diode {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        dio.linear_stamp(&nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }

    #[test]
    fn test_count_nonlinear_funcs() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(dio.count_nonlinear_funcs(), g.len());
    }

    #[test]
    fn test_nonlinear_funcs_node_0_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("0"), String::from("1")],
        };
        let nodes = parse_dio(&dio);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, [[-1.0]]);
        assert_eq!(g.len(), 1);

        let x_test: Vec<f64> = vec![1.5, 1.0];
        assert!(g[0](&x_test) < 0.0);
    }

    #[test]
    fn test_nonlinear_funcs_node_1_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("0")],
        };
        let nodes = parse_dio(&dio);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        assert_eq!(h, [[1.0]]);
        assert_eq!(g.len(), 1);

        let x_test: Vec<f64> = vec![1.5, 1.0];
        assert!(g[0](&x_test) > 0.0);
    }

    #[test]
    fn test_nonlinear_funcs_diode_two_nodes() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        dio.nonlinear_funcs(&nodes, &mut h, &mut g);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        let mut h_model = vec![vec![0.0; 1]; 2];
        h_model[n1][0] = 1.0;
        h_model[n2][0] = -1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 1);

        let mut x_test: Vec<f64> = vec![0.0; 2];
        x_test[n1] = 1.5;
        x_test[n2] = 1.0;

        assert!(g[0](&x_test) > 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_node_0_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("0"), String::from("1")],
        };
        let nodes = parse_dio(&dio);
        let x: Vec<f64> = vec![1.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[0][0] > 0.0);
        assert!(b[0] < 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_node_1_gnd() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("0")],
        };
        let nodes = parse_dio(&dio);
        let x: Vec<f64> = vec![1.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        assert!(a[0][0] > 0.0);
        assert!(b[0] > 0.0);
    }

    #[test]
    fn test_nonlinear_stamp_two_nodes() {
        let dio = Diode {
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
        };
        let nodes = parse_dio(&dio);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        dio.nonlinear_stamp(&nodes, &x, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        assert!(a[n1][n1] > 0.0);
        assert!(a[n1][n2] < 0.0);
        assert!(a[n2][n1] < 0.0);
        assert!(a[n2][n2] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] < 0.0);
    }
}
