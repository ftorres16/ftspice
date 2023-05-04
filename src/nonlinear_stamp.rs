use std::collections::BTreeMap;

use crate::device;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::NodeType>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    match elem.dtype {
        device::DType::Vdd => {}
        device::DType::Idd => {}
        device::DType::Res => {}
        device::DType::Diode => load_diode(elem, nodes, x, a, b),
    }
}

fn load_diode(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::NodeType>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let vpos_idx = nodes.keys().position(|x| x == &elem.nodes[0]);
    let vneg_idx = nodes.keys().position(|x| x == &elem.nodes[1]);

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
