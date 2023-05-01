use std::collections::BTreeSet;

#[derive(Debug)]
pub enum DType {
    Vdd,
    Idd,
    Res,
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
                    .unwrap();

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
        };
    }
}
