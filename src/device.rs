use std::collections::BTreeSet;

mod diode;

#[derive(Debug)]
pub enum DType {
    Vdd,
    Idd,
    Res,
    Diode,
}

#[derive(Debug)]
pub struct SpiceElem {
    pub dtype: DType,
    pub name: String,
    pub nodes: Vec<String>,
    pub value: f64,
}

impl SpiceElem {
    pub fn linear_stamp(&self, nodes: &BTreeSet<String>, a: &mut Vec<Vec<f64>>, b: &mut Vec<f64>) {
        match self.dtype {
            DType::Vdd => {
                let vneg_idx = nodes.iter().position(|x| x.to_string() == self.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == self.nodes[1]);
                let is_idx = nodes
                    .iter()
                    .position(|x| x.to_string() == self.name)
                    .expect("Couldn't find matrix entry for source.");

                b[is_idx] += self.value;

                if let Some(i) = vpos_idx {
                    a[is_idx][i] += 1.0;
                    a[i][is_idx] += 1.0;
                }

                if let Some(i) = vneg_idx {
                    a[is_idx][i] -= 1.0;
                    a[i][is_idx] -= 1.0;
                }
            }
            DType::Idd => {
                let vneg_idx = nodes.iter().position(|x| x.to_string() == self.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == self.nodes[1]);

                if let Some(i) = vpos_idx {
                    b[i] += self.value;
                }
                if let Some(i) = vneg_idx {
                    b[i] -= self.value;
                }
            }
            DType::Res => {
                let g = 1.0 / self.value;

                let vneg_idx = nodes.iter().position(|x| x.to_string() == self.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == self.nodes[1]);

                if let Some(i) = vneg_idx {
                    a[i][i] += g;
                }
                if let Some(i) = vpos_idx {
                    a[i][i] += g;
                }
                if let (Some(i), Some(j)) = (vpos_idx, vneg_idx) {
                    a[i][j] -= g;
                    a[j][i] -= g;
                }
            }
            DType::Diode => {
                unimplemented!("Diodes not implemented yet!");
            }
        };
    }

    pub fn taylor_stamp(
        &self,
        nodes: &BTreeSet<String>,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    ) {
        match self.dtype {
            DType::Vdd => {}
            DType::Idd => {}
            DType::Res => {}
            DType::Diode => {
                let vpos_idx = nodes.iter().position(|x| x.to_string() == self.nodes[0]);
                let vneg_idx = nodes.iter().position(|x| x.to_string() == self.nodes[1]);

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
    }
}
