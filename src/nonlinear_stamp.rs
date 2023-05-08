use std::collections::HashMap;

use crate::device;
use crate::node;

pub fn load(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    match elem.dtype {
        device::DType::Vdd => {}
        device::DType::Idd => {}
        device::DType::Res => {}
        device::DType::Diode => load_diode(elem, nodes, x, a, b),
        device::DType::NPN => load_npn(elem, nodes, x, a, b),
        device::DType::NMOS => load_nmos(elem, nodes, x, a, b),
    }
}

fn load_diode(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let vpos_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vneg_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);

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

fn load_npn(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let vc_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vb_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let ve_idx = nodes.get(&elem.nodes[2]).map(|x| x.idx);

    let vc = match vc_idx {
        Some(i) => x[i],
        None => 0.0,
    };
    let vb = match vb_idx {
        Some(i) => x[i],
        None => 0.0,
    };
    let ve = match ve_idx {
        Some(i) => x[i],
        None => 0.0,
    };

    let q = device::npn::NPN {
        vc: vc,
        vb: vb,
        ve: ve,
    };

    let gee = q.gee();
    let gec = q.gec();
    let gce = q.gce();
    let gcc = q.gcc();
    let i_e = q.ie_eq();
    let i_c = q.ic_eq();

    if let Some(i) = vc_idx {
        a[i][i] += gcc;
        b[i] -= i_c;
    }
    if let Some(i) = vb_idx {
        a[i][i] += gcc + gee - gce - gec;
        b[i] += i_e + i_c;
    }
    if let Some(i) = ve_idx {
        a[i][i] += gee;
        b[i] -= i_e;
    }
    if let (Some(i), Some(j)) = (ve_idx, vc_idx) {
        a[i][j] -= gec;
        a[j][i] -= gce;
    }
    if let (Some(i), Some(j)) = (ve_idx, vb_idx) {
        a[i][j] += gec - gee;
        a[j][i] += gce - gee;
    }
    if let (Some(i), Some(j)) = (vc_idx, vb_idx) {
        a[i][j] += gce - gcc;
        a[j][i] += gec - gcc;
    }
}

fn load_nmos(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    x: &Vec<f64>,
    a: &mut Vec<Vec<f64>>,
    b: &mut Vec<f64>,
) {
    let mut vd_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vg_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let mut vs_idx = nodes.get(&elem.nodes[2]).map(|x| x.idx);

    let mut vd = match vd_idx {
        Some(i) => x[i],
        None => 0.0,
    };
    let vg = match vg_idx {
        Some(i) => x[i],
        None => 0.0,
    };
    let mut vs = match vs_idx {
        Some(i) => x[i],
        None => 0.0,
    };

    if vs > vd {
        (vd, vs) = (vs, vd);
        (vd_idx, vs_idx) = (vs_idx, vd_idx);
    }

    let m = device::nmos::NMOS {
        vd: vd,
        vg: vg,
        vs: vs,
    };

    let gds = m.gds();
    let gm = m.gm();
    let ieq = m.ieq();

    if let Some(i) = vd_idx {
        a[i][i] += gds;
        b[i] -= ieq;
    }
    if let Some(i) = vs_idx {
        a[i][i] += gds + gm;
        b[i] += ieq;
    }
    if let (Some(i), Some(j)) = (vd_idx, vs_idx) {
        a[i][j] -= gds + gm;
        a[j][i] -= gds;
    }
    if let (Some(i), Some(j)) = (vd_idx, vg_idx) {
        a[i][j] += gm;
    }
    if let (Some(i), Some(j)) = (vs_idx, vg_idx) {
        a[i][j] -= gm;
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
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
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let x: Vec<f64> = vec![1.0, 2.0];
        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 2]; 2];
        let mut b: Vec<f64> = vec![0.0; 2];

        load_diode(&elem, &nodes, &x, &mut a, &mut b);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;

        assert!(a[n1][n1] > 0.0);
        assert!(a[n1][n2] < 0.0);
        assert!(a[n2][n1] < 0.0);
        assert!(a[n2][n2] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] < 0.0);
    }

    #[test]
    fn test_load_npn_three_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::NPN,
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
            value: None,
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;
        let n3 = nodes.get("3").unwrap().idx;

        let mut x: Vec<f64> = vec![0.0; 3];
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        load_npn(&elem, &nodes, &x, &mut a, &mut b);

        assert!(a[n1][n1] > 0.0);
        assert!(a[n2][n2] > 0.0);
        assert!(a[n3][n3] > 0.0);
        assert!(b[n1] > 0.0);
        assert!(b[n2] > 0.0);
        assert!(b[n3] < 0.0);
    }

    #[test]
    fn test_load_nmos_three_nodes() {
        let elem = device::SpiceElem {
            dtype: device::DType::NMOS,
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
            value: None,
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;
        let n3 = nodes.get("3").unwrap().idx;

        let mut x: Vec<f64> = vec![0.0; 3];
        x[n1] = 2.0;
        x[n2] = 1.0;
        x[n3] = 0.0;

        let mut a: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut b: Vec<f64> = vec![0.0; 3];

        load_nmos(&elem, &nodes, &x, &mut a, &mut b);

        assert!(a[n1][n1] > 0.0);
        assert!(a[n1][n2] > 0.0);
        assert!(a[n1][n3] < 0.0);
        assert_eq!(a[n2], [0.0, 0.0, 0.0]);
        assert!(a[n3][n1] < 0.0);
        assert!(a[n3][n2] < 0.0);
        assert!(a[n3][n3] > 0.0);

        assert!(b[n1] > 0.0);
        assert_eq!(b[n2], 0.0);
        assert!(b[n3] < 0.0);
    }
}
