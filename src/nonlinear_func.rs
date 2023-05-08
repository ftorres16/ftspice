use std::collections::HashMap;

use crate::device;
use crate::node;

pub fn count(elem: &device::SpiceElem) -> usize {
    match elem.dtype {
        device::DType::Vdd => 0,
        device::DType::Idd => 0,
        device::DType::Res => 0,
        device::DType::Diode => 1,
        device::DType::NPN => 3,
        device::DType::NMOS => 3,
    }
}

pub fn load(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    h_mat: &mut Vec<Vec<f64>>,
    g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) {
    match elem.dtype {
        device::DType::Vdd => {}
        device::DType::Idd => {}
        device::DType::Res => {}
        device::DType::Diode => load_diode(elem, nodes, h_mat, g_vec),
        device::DType::NPN => load_npn(elem, nodes, h_mat, g_vec),
        device::DType::NMOS => load_nmos(elem, nodes, h_mat, g_vec),
    }
}

fn load_diode(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    h_mat: &mut Vec<Vec<f64>>,
    g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) {
    let vpos_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vneg_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);

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

fn load_npn(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    h_mat: &mut Vec<Vec<f64>>,
    g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) {
    let vc_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vb_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let ve_idx = nodes.get(&elem.nodes[2]).map(|x| x.idx);

    if let Some(i) = vc_idx {
        h_mat[i][g_vec.len()] = 1.0;
    }
    if let Some(i) = vb_idx {
        h_mat[i][g_vec.len() + 1] = 1.0;
    }
    if let Some(i) = ve_idx {
        h_mat[i][g_vec.len() + 2] = 1.0;
    }

    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
        q.ic()
    }));
    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
        q.ib()
    }));
    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
        q.ie()
    }));
}

fn load_nmos(
    elem: &device::SpiceElem,
    nodes: &HashMap<String, node::MNANode>,
    h_mat: &mut Vec<Vec<f64>>,
    g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) {
    let vd_idx = nodes.get(&elem.nodes[0]).map(|x| x.idx);
    let vg_idx = nodes.get(&elem.nodes[1]).map(|x| x.idx);
    let vs_idx = nodes.get(&elem.nodes[2]).map(|x| x.idx);

    if let Some(i) = vd_idx {
        h_mat[i][g_vec.len()] = 1.0;
    }
    if let Some(i) = vg_idx {
        h_mat[i][g_vec.len() + 1] = 1.0;
    }
    if let Some(i) = vs_idx {
        h_mat[i][g_vec.len() + 2] = 1.0;
    }

    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
            (vs, vd) = (vd, vs);
        }

        let m = device::nmos::NMOS {
            vd: vd,
            vg: vg,
            vs: vs,
        };
        m.id()
    }));
    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
            (vs, vd) = (vd, vs);
        }

        let m = device::nmos::NMOS {
            vd: vd,
            vg: vg,
            vs: vs,
        };
        m.ig()
    }));
    g_vec.push(Box::new(move |x: &Vec<f64>| {
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
            (vs, vd) = (vd, vs);
        }

        let m = device::nmos::NMOS {
            vd: vd,
            vg: vg,
            vs: vs,
        };
        m.is()
    }));
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
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        load_diode(&elem, &nodes, &mut h, &mut g);

        assert_eq!(h, [[-1.0]]);
        assert_eq!(g.len(), 1);

        let x_test: Vec<f64> = vec![1.5, 1.0];
        assert!(g[0](&x_test) < 0.0);
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
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 1];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        load_diode(&elem, &nodes, &mut h, &mut g);

        assert_eq!(h, [[1.0]]);
        assert_eq!(g.len(), 1);

        let x_test: Vec<f64> = vec![1.5, 1.0];
        assert!(g[0](&x_test) > 0.0);
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
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 1]; 2];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        load_diode(&elem, &nodes, &mut h, &mut g);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;

        let mut h_model = vec![vec![0.0; 1]; 2];
        h_model[n1][0] = 1.0;
        h_model[n2][0] = -1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 1);

        let mut x_test: Vec<f64> = vec![0.0; 2];
        x_test[n1] = 1.5;
        x_test[n2] = 1.0;

        assert!(g[0](&x_test) > 0.0);
    }

    #[test]
    fn test_load_npn() {
        let elem = device::SpiceElem {
            dtype: device::DType::NPN,
            name: String::from("Q1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
            value: None,
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        load_npn(&elem, &nodes, &mut h, &mut g);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;
        let n3 = nodes.get("3").unwrap().idx;

        let mut h_model = vec![vec![0.0; h[0].len()]; h.len()];
        h_model[n1][0] = 1.0;
        h_model[n2][1] = 1.0;
        h_model[n3][2] = 1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 3);

        let mut x_test: Vec<f64> = vec![0.0; 3];
        x_test[n1] = 2.0;
        x_test[n2] = 1.0;
        x_test[n3] = 0.0;

        assert!(g[0](&x_test) > 0.0);
        assert!(g[1](&x_test) > 0.0);
        assert!(g[2](&x_test) < 0.0);
    }

    #[test]
    fn test_load_nmos() {
        let elem = device::SpiceElem {
            dtype: device::DType::NMOS,
            name: String::from("M1"),
            nodes: vec![String::from("1"), String::from("2"), String::from("3")],
            value: None,
        };
        let nodes = node::parse_elems(&vec![elem.clone()]);
        let mut h: Vec<Vec<f64>> = vec![vec![0.0; 3]; 3];
        let mut g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

        load_nmos(&elem, &nodes, &mut h, &mut g);

        let n1 = nodes.get("1").unwrap().idx;
        let n2 = nodes.get("2").unwrap().idx;
        let n3 = nodes.get("3").unwrap().idx;

        let mut h_model = vec![vec![0.0; h[0].len()]; h.len()];
        h_model[n1][0] = 1.0;
        h_model[n2][1] = 1.0;
        h_model[n3][2] = 1.0;

        assert_eq!(h, h_model);
        assert_eq!(g.len(), 3);

        let mut x_test: Vec<f64> = vec![0.0; 3];
        x_test[n1] = 2.0;
        x_test[n2] = 1.0;
        x_test[n3] = 0.0;

        assert!(g[0](&x_test) > 0.0);
        assert_eq!(g[1](&x_test), 0.0);
        assert!(g[2](&x_test) < 0.0);
    }
}
