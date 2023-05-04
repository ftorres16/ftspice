use std::collections::BTreeMap;

use crate::device;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::NodeType>,
    h_mat: &mut Vec<Vec<f64>>,
    g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) {
    match elem.dtype {
        device::DType::Vdd => {}
        device::DType::Idd => {}
        device::DType::Res => {}
        device::DType::Diode => {
            let vpos_idx = nodes.keys().position(|x| x == &elem.nodes[0]);
            let vneg_idx = nodes.keys().position(|x| x == &elem.nodes[1]);

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
                let d = device::diode::Diode {
                    vpos: vpos,
                    vneg: vneg,
                };
                d.i()
            }));
        }
    }
}
