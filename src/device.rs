use std::collections::BTreeMap;

mod diode;

#[derive(Debug)]
pub enum DType {
    Vdd,
    Idd,
    Res,
    Diode,
}

#[derive(Debug)]
pub enum NodeType {
    G1,
    G2,
}

#[derive(Debug)]
pub struct SpiceElem {
    pub dtype: DType,
    pub name: String,
    pub nodes: Vec<String>,
    pub value: Option<f64>,
}

impl SpiceElem {
    pub fn num_nonlinear_funcs(&self) -> usize {
        match self.dtype {
            DType::Vdd => 0,
            DType::Idd => 0,
            DType::Res => 0,
            DType::Diode => 1,
        }
    }

    pub fn nonlinear_func(
        &self,
        nodes: &BTreeMap<String, NodeType>,
        h_mat: &mut Vec<Vec<f64>>,
        g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
        match self.dtype {
            DType::Vdd => {}
            DType::Idd => {}
            DType::Res => {}
            DType::Diode => {
                let vpos_idx = nodes.keys().position(|x| x == &self.nodes[0]);
                let vneg_idx = nodes.keys().position(|x| x == &self.nodes[1]);

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
                    let d = diode::Diode {
                        vpos: vpos,
                        vneg: vneg,
                    };
                    d.i()
                }));
            }
        }
    }

    pub fn taylor_stamp(
        &self,
        nodes: &BTreeMap<String, NodeType>,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        match self.dtype {
            DType::Vdd => {}
            DType::Idd => {}
            DType::Res => {}
            DType::Diode => self.diode_stamp(nodes, x, a, b),
        }
    }

    fn diode_stamp(
        &self,
        nodes: &BTreeMap<String, NodeType>,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        let vpos_idx = nodes.keys().position(|x| x == &self.nodes[0]);
        let vneg_idx = nodes.keys().position(|x| x == &self.nodes[1]);

        let vpos = match vpos_idx {
            Some(i) => x[i],
            None => 0.0,
        };
        let vneg = match vneg_idx {
            Some(i) => x[i],
            None => 0.0,
        };

        let d = diode::Diode {
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
