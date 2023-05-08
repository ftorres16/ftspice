use std::collections::HashMap;

use crate::device;
use crate::node;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
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
    nodes: &HashMap<String, node::MNANode>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let vneg_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vpos_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let is_idx = nodes
        .get(&elem.name)
        .expect("Couldn't find matrix entry for source.")
        .idx;

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

fn load_idd(elem: &device::SpiceElem, nodes: &HashMap<String, node::MNANode>, b: &mut Vec<f64>) {
    let vneg_node = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vpos_node = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let val = elem.value.expect("Current source has no value");

    if let Some(i) = vpos_node {
        b[i] += val;
    }
    if let Some(i) = vneg_node {
        b[i] -= val;
    }
}

fn load_res(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    a: &mut Vec<Vec<f64>>,
) {
    let g = 1.0 / elem.value.expect("Res has no value");

    let vneg_node = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vpos_node = nodes.get(&elem.nodes[1]).map(|x| x.idx);

    if let Some(i) = vneg_node {
        a[i][i] += g;
    }
    if let Some(i) = vpos_node {
        a[i][i] += g;
    }
    if let (Some(i), Some(j)) = (vpos_node, vneg_node) {
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let v1 = nodes.get("V1").unwrap().idx;

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = 1.0;
        a_model[v1][n1] = 1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_vdd_node_1_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Vdd,
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: Some(1e-3),
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let v1 = nodes.get("V1").unwrap().idx;
        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = -1.0;
        a_model[v1][n1] = -1.0;
        b_model[n1] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_vdd_to_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::Vdd,
            name: String::from("V1"),
            nodes: vec![String::from("1"), String::from("2")],
            value: Some(1e-3),
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        load_vdd(&elem, &nodes, &mut a, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;
        let v1 = nodes.get("V1").unwrap().idx;

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b_model = vec![0.0; nodes.len()];
        a_model[n1][v1] = -1.0;
        a_model[v1][n1] = -1.0;
        a_model[n2][v1] = 1.0;
        a_model[v1][n2] = 1.0;
        b_model[n1] = 0.0;
        b_model[n2] = 0.0;
        b_model[v1] = 1e-3;

        assert_eq!(a, a_model);
        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_idd_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Idd,
            name: String::from("I1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: Some(1e-3),
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut b: Vec<f64> = vec![0.0; 2];

        load_idd(&elem, &nodes, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;

        let mut b_model = vec![0.0; nodes.len()];
        b_model[n1] = -1e-3;
        b_model[n2] = 1e-3;

        assert_eq!(b, b_model);
    }

    #[test]
    fn test_load_res_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Res,
            name: String::from("R1"),
            nodes: vec![String::from("0"), String::from("1")],
            value: Some(1e3),
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];

        load_res(&elem, &nodes, &mut a);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;

        let mut a_model = vec![vec![0.0; nodes.len()]; nodes.len()];
        a_model[n1][n1] = 1e-3;
        a_model[n1][n2] = -1e-3;
        a_model[n2][n1] = -1e-3;
        a_model[n2][n2] = 1e-3;

        assert_eq!(a, a_model);
    }
}
