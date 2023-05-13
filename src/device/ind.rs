use crate::device::{GType, Stamp};
use crate::node::NodeCollection;

mod model;

#[derive(Debug, Clone)]
pub struct Ind {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
    pub u_curr: Option<f64>,
    pub i_curr: Option<f64>,
}

impl Stamp for Ind {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> GType {
        GType::G1
    }

    fn gtype_startup(&self) -> GType {
        GType::G2
    }

    fn get_value(&self) -> f64 {
        self.val
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn init_state(&mut self, nodes: &NodeCollection, x: &Vec<f64>) {
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find label for inductor");

        self.u_curr = Some(0.0);
        self.i_curr = Some(x[is_idx]);
    }

    fn update_state(&mut self, nodes: &NodeCollection, x: &Vec<f64>, h: &f64) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let l = model::Model {
            vpos,
            vneg,
            val: self.val,
            u_old: self.u_curr.expect("Ind voltage history not initialized"),
            i_old: self.i_curr.expect("Ind current history not initialized"),
        };

        self.u_curr = Some(l.u_new());
        self.i_curr = Some(l.i_new(h));
    }

    fn linear_startup_stamp(
        &self,
        nodes: &NodeCollection,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for inductor");

        b[is_idx] = 0.0;

        if let Some(i) = vpos_idx {
            a[is_idx][i] += 1.0;
            a[i][is_idx] += 1.0;
        }
        if let Some(i) = vneg_idx {
            a[is_idx][i] -= 1.0;
            a[i][is_idx] -= 1.0;
        }
    }

    fn undo_linear_startup_stamp(
        &self,
        nodes: &NodeCollection,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);
        let is_idx = nodes
            .get_idx(&self.name)
            .expect("Couldn't find node label for inductor");

        b[is_idx] = 0.0;

        if let Some(i) = vpos_idx {
            a[is_idx][i] -= 1.0;
            a[i][is_idx] -= 1.0;
        }
        if let Some(i) = vneg_idx {
            a[is_idx][i] -= 1.0;
            a[i][is_idx] -= 1.0;
        }
    }

    fn dynamic_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Vec<f64>,
        h: &f64,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let l = model::Model {
            vpos,
            vneg,
            val: self.val,
            u_old: self.u_curr.expect("Cap voltage history not initialized"),
            i_old: self.i_curr.expect("Cap current history not initialized"),
        };

        let g_eq = l.g_eq(h);
        let i_eq = l.i_eq(h);

        if let Some(i) = vpos_idx {
            a[i][i] += g_eq;
            b[i] -= i_eq;
        }
        if let Some(i) = vneg_idx {
            a[i][i] += g_eq;
            b[i] += i_eq;
        }
        if let (Some(i), Some(j)) = (vneg_idx, vpos_idx) {
            a[i][j] -= g_eq;
            a[j][i] -= g_eq;
        }
    }

    fn undo_dynamic_stamp(
        &self,
        nodes: &NodeCollection,
        x: &Vec<f64>,
        h: &f64,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.get_idx(&self.nodes[0]);
        let vneg_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        let l = model::Model {
            vpos,
            vneg,
            val: self.val,
            u_old: self.u_curr.expect("Cap voltage history not initialized"),
            i_old: self.i_curr.expect("Cap current history not initialized"),
        };

        let g_eq = l.g_eq(h);
        let i_eq = l.i_eq(h);

        if let Some(i) = vpos_idx {
            a[i][i] -= g_eq;
            b[i] += i_eq;
        }
        if let Some(i) = vneg_idx {
            a[i][i] -= g_eq;
            b[i] -= i_eq;
        }
        if let (Some(i), Some(j)) = (vneg_idx, vpos_idx) {
            a[i][j] += g_eq;
            a[j][i] += g_eq;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ind(ind: &Ind) -> NodeCollection {
        NodeCollection::from_elems(&vec![Box::new(ind.clone())])
    }

    fn test_ind(nodes: &[&str]) -> Ind {
        Ind {
            name: String::from("L1"),
            nodes: nodes.iter().map(|s| s.to_string()).collect(),
            val: 1e-3,
            u_curr: Some(0.0),
            i_curr: Some(1e-3),
        }
    }

    #[test]
    fn test_dynamic_stamp() {
        let ind = test_ind(&["1", "2"]);
        let nodes = parse_ind(&ind);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];
        let x = vec![1.0, 2.0];
        let h = 1e-8;

        ind.dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);

        let n1 = nodes.get_idx("1").unwrap();
        let n2 = nodes.get_idx("2").unwrap();

        assert!(a[n1][n1] > 0.0);
        assert!(a[n1][n2] < 0.0);
        assert!(a[n2][n1] < 0.0);
        assert!(a[n2][n2] > 0.0);

        assert!(b[n1] < 0.0);
        assert!(b[n2] > 0.0);
    }

    #[test]
    fn test_undo_dynamic_stamp() {
        let ind = test_ind(&["1", "2"]);
        let nodes = parse_ind(&ind);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];
        let x = vec![1.0, 2.0];
        let h = 1e-8;

        ind.dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);
        ind.undo_dynamic_stamp(&nodes, &x, &h, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0]);
    }
}
