use crate::device::{GType, Stamp};
use crate::node::NodeCollection;

mod model;

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

    fn init_state(&mut self, nodes: &NodeCollection, x: &Vec<f64>) {
        let vneg_idx = nodes.get_idx(&self.nodes[0]);
        let vpos_idx = nodes.get_idx(&self.nodes[1]);

        let vpos = vpos_idx.map_or(0.0, |i| x[i]);
        let vneg = vneg_idx.map_or(0.0, |i| x[i]);

        self.u_curr = Some(vpos - vneg);
        self.i_curr = Some(0.0);
    }

    fn update_state(&mut self, nodes: &NodeCollection, x: &Vec<f64>, h: &f64) {
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
        x: &Vec<f64>,
        h: &f64,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
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
