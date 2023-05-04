use std::collections::BTreeMap;

use crate::device;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::RowType>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    match elem.dtype {
        device::DType::Vdd => load_vdd(elem, nodes, a, b),
        device::DType::Idd => load_idd(elem, nodes, b),
        device::DType::Res => load_res(elem, nodes, a),
        device::DType::Diode => (),
    };
}

fn load_vdd(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::RowType>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let vneg_idx = nodes.keys().position(|x| x == &elem.nodes[0]);
    let vpos_idx = nodes.keys().position(|x| x == &elem.nodes[1]);
    let is_idx = nodes
        .keys()
        .position(|x| x == &elem.name)
        .expect("Couldn't find matrix entry for source.");

    b[is_idx] += elem.value.expect("Voltage source has no value");

    if let Some(i) = vpos_idx {
        a[is_idx][i] += 1.0;
        a[i][is_idx] += 1.0;
    }

    if let Some(i) = vneg_idx {
        a[is_idx][i] -= 1.0;
        a[i][is_idx] -= 1.0;
    }
}

fn load_idd(elem: &device::SpiceElem, nodes: &BTreeMap<String, device::RowType>, b: &mut Vec<f64>) {
    let vneg_idx = nodes.keys().position(|x| x == &elem.nodes[0]);
    let vpos_idx = nodes.keys().position(|x| x == &elem.nodes[1]);
    let val = elem.value.expect("Current source has no value");

    if let Some(i) = vpos_idx {
        b[i] += val;
    }
    if let Some(i) = vneg_idx {
        b[i] -= val;
    }
}

fn load_res(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::RowType>,
    a: &mut Vec<Vec<f64>>,
) {
    let g = 1.0 / elem.value.expect("Res has no value");

    let vneg_idx = nodes.keys().position(|x| x == &elem.nodes[0]);
    let vpos_idx = nodes.keys().position(|x| x == &elem.nodes[1]);

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
