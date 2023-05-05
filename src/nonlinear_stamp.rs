use std::collections::BTreeMap;

use crate::device;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &BTreeMap<String, device::RowType>,
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
    nodes: &BTreeMap<String, device::RowType>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_diode_node_0_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Diode,
            name: String::from("D1"),
            nodes: vec![String::from("0"), String::from("1")],
            value: None,
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let x: Vec<f64> = vec![1.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        load_diode(&elem, &nodes, &x, &mut a, &mut b);

        assert!(a[0][0] > 0.0);
        assert!(b[0] < 0.0);
    }

    #[test]
    fn test_load_diode_node_1_gnd() {
        let elem = device::SpiceElem {
            dtype: device::DType::Diode,
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("0")],
            value: None,
        };
        let nodes = BTreeMap::from([(String::from("1"), device::RowType::Voltage)]);
        let x: Vec<f64> = vec![1.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut b: Vec<f64> = vec![0.0; 1];

        load_diode(&elem, &nodes, &x, &mut a, &mut b);

        assert!(a[0][0] > 0.0);
        assert!(b[0] > 0.0);
    }

    #[test]
    fn test_load_diode_two_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::Diode,
            name: String::from("D1"),
            nodes: vec![String::from("1"), String::from("2")],
            value: None,
        };
        let nodes = BTreeMap::from([
            (String::from("1"), device::RowType::Voltage),
            (String::from("2"), device::RowType::Voltage),
        ]);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_diode(&elem, &nodes, &x, &mut a, &mut b);

        assert!(a[0][0] > 0.0);
        assert!(a[0][1] < 0.0);
        assert!(a[1][0] < 0.0);
        assert!(a[1][1] > 0.0);
        assert!(b[0] > 0.0);
        assert!(b[1] < 0.0);
    }
}
