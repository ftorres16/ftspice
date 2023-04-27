use std::collections::BTreeSet;

mod gauss_lu;

const GND: &str = "0";

#[derive(Debug)]
enum DeviceType {
    Vdd,
    Idd,
    Res,
}

#[derive(Debug)]
struct SpiceElem {
    dtype: DeviceType,
    name: String,
    nodes: Vec<String>,
    value: f64,
}

fn main() {
    let mut elems: Vec<SpiceElem> = Vec::new();

    elems.push(SpiceElem {
        dtype: DeviceType::Vdd,
        name: "V0".to_string(),
        nodes: vec!["0".to_string(), "1".to_string()],
        value: 3.0,
    });
    elems.push(SpiceElem {
        dtype: DeviceType::Res,
        name: "R1".to_string(),
        nodes: vec!["1".to_string(), "2".to_string()],
        value: 1e3,
    });
    elems.push(SpiceElem {
        dtype: DeviceType::Res,
        name: "R1".to_string(),
        nodes: vec!["2".to_string(), "0".to_string()],
        value: 1e3,
    });

    let nodes = find_nodes(&elems);

    let mut a_mat = vec![vec![0.0; nodes.len()]; nodes.len()];
    let mut b_vec = vec![0.0; nodes.len()];
    let mut x_vec = vec![0.0; nodes.len()];

    load_stamps(&elems, &nodes, &mut a_mat, &mut b_vec);

    gauss_lu::solve(&mut a_mat, &mut b_vec, &mut x_vec);

    for (node, val) in nodes.iter().zip(x_vec.iter()) {
        println!("{node}: {val}");
    }
}

fn find_nodes(elems: &Vec<SpiceElem>) -> BTreeSet<String> {
    let mut nodes: BTreeSet<String> = BTreeSet::new();

    for elem in elems.iter() {
        for node in elem.nodes.iter() {
            nodes.insert(node.to_string());
        }

        if let DeviceType::Vdd = elem.dtype {
            nodes.insert(elem.name.to_string());
        }
    }

    if !nodes.contains(GND) {
        panic!("GND not found!");
    }
    nodes.remove(GND);

    nodes
}

fn load_stamps(
    elems: &Vec<SpiceElem>,
    nodes: &BTreeSet<String>,
    a_mat: &mut Vec<Vec<f64>>,
    b_vec: &mut Vec<f64>,
) -> () {
    for elem in elems.iter() {
        match elem.dtype {
            DeviceType::Vdd => {
                let vneg_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[1]);
                let is_idx = nodes
                    .iter()
                    .position(|x| x.to_string() == elem.name)
                    .unwrap();

                b_vec[is_idx] += elem.value;

                if let Some(i) = vpos_idx {
                    a_mat[is_idx][i] += 1.0;
                    a_mat[i][is_idx] += 1.0;
                }

                if let Some(i) = vneg_idx {
                    a_mat[is_idx][i] -= 1.0;
                    a_mat[i][is_idx] -= 1.0;
                }
            }
            DeviceType::Idd => {
                let vneg_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[1]);

                if let Some(i) = vpos_idx {
                    b_vec[i] += elem.value;
                }
                if let Some(i) = vneg_idx {
                    b_vec[i] -= elem.value;
                }
            }
            DeviceType::Res => {
                let g = 1.0 / elem.value;

                let vneg_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[0]);
                let vpos_idx = nodes.iter().position(|x| x.to_string() == elem.nodes[1]);

                if let Some(i) = vneg_idx {
                    a_mat[i][i] += g;
                }
                if let Some(i) = vpos_idx {
                    a_mat[i][i] += g;
                }
                if let (Some(i), Some(j)) = (vpos_idx, vneg_idx) {
                    a_mat[i][j] -= g;
                    a_mat[j][i] -= g;
                }
            }
        }
    }
}
