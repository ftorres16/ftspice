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
        device::DType::NPN => (),
        device::DType::NMOS => (),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_vdd_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Vdd,
            name: String::from("V1"),
            nodes: vec![String::from("0"), String::from("1")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("V1"), device::RowType::Current),
        ]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 1.0], [1.0, 0.0]]);
        assert_eq!(b, [0.0, 1e-3]);
    }

    #[test]
    fn test_load_vdd_node_1_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Vdd,
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("V1"), device::RowType::Current),
        ]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, -1.0], [-1.0, 0.0]]);
        assert_eq!(b, [0.0, 1e-3]);
    }

    #[test]
    fn test_load_vdd_to_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::Vdd,
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("2"), device::RowType::Voltage),
            (String::from("V1"), device::RowType::Current),
        ]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        assert_eq!(a, [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0], [-1.0, 1.0, 0.0]]);
        assert_eq!(b, [0.0, 0.0, 1e-3]);
    }

    #[test]
    fn test_load_idd_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Idd,
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let mut b: Vec<f64> = vec![0.0; 1];

        load_idd(&elem, &nodes, &mut b);

        assert_eq!(b, [-1e-3]);
    }

    #[test]
    fn test_load_idd_node_1_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Idd,
            name: String::from("I1"),
            nodes: vec![String::from("0"), String::from("1")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let mut b: Vec<f64> = vec![0.0; 1];

        load_idd(&elem, &nodes, &mut b);

        assert_eq!(b, [1e-3]);
    }

    #[test]
    fn test_load_idd_to_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::Idd,
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("2")],
            value: Some(1e-3),
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("2"), device::RowType::Voltage),
        ]);
        let mut b: Vec<f64> = vec![0.0; 2];

        load_idd(&elem, &nodes, &mut b);

        assert_eq!(b, [-1e-3, 1e-3]);
    }

    #[test]
    fn test_load_res_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Res,
            name: String::from("R1"),
            nodes: vec![String::from("0"), String::from("1")],
            value: Some(1e3),
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]];

        load_res(&elem, &nodes, &mut a);

        assert_eq!(a, [[1e-3]]);
    }

    #[test]
    fn test_load_res_node_1_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Res,
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: Some(1e3),
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]];

        load_res(&elem, &nodes, &mut a);

        assert_eq!(a, [[1e-3]]);
    }

    #[test]
    fn test_load_res_to_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::Res,
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            value: Some(1e3),
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("2"), device::RowType::Voltage),
        ]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];

        load_res(&elem, &nodes, &mut a);

        assert_eq!(a, [[1e-3, -1e-3], [-1e-3, 1e-3]]);
    }
}
