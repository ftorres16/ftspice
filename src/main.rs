use std::collections::BTreeMap;
use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod command;
mod device;
mod gauss_lu;
mod linalg;
mod linear_stamp;
mod newtons_method;
mod nonlinear_func;
mod nonlinear_stamp;
mod parser;

const GND: &str = "0";

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args
        .get(1)
        .expect("Insufficient arguments. Specify spice file to simulate.");

    let (elems, cmds) = parser::parse_spice_file(file);

    let nodes = find_nodes(&elems);
    let num_nonlinear_funcs = elems.iter().map(|x| nonlinear_func::count(x)).sum();

    let mut x_vec = vec![0.0; nodes.len()];

    // Linear elements
    let mut a_mat = vec![vec![0.0; nodes.len()]; nodes.len()];
    let mut b_vec = vec![0.0; nodes.len()];

    // Nonlinear elements
    let mut h_mat = vec![vec![0.0; num_nonlinear_funcs]; nodes.len()];
    let mut g_vec: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

    for elem in elems.iter() {
        linear_stamp::load(&elem, &nodes, &mut a_mat, &mut b_vec);
        nonlinear_func::load(&elem, &nodes, &mut h_mat, &mut g_vec);
    }

    if let Some(_) = cmds.iter().find(|x| matches!(x, command::Command::Op)) {
        let n_iters =
            newtons_method::solve(&nodes, &elems, &mut x_vec, &a_mat, &b_vec, &h_mat, &g_vec);

        if cmds.len() == 1 {
            // Print OP results only if it's the only analysis
            print_headers(&nodes);
            print_results(n_iters, &x_vec);
        }
    }

    if let Some(command::Command::DC(dc_params)) =
        cmds.iter().find(|x| matches!(x, command::Command::DC(_)))
    {
        let src_idx = nodes
            .keys()
            .position(|x| x == &dc_params.source)
            .expect("Sweep source not found!");

        let mut sweep_val = dc_params.start;

        print_headers(&nodes);

        while sweep_val < dc_params.stop {
            b_vec[src_idx] = sweep_val;

            let n_iters =
                newtons_method::solve(&nodes, &elems, &mut x_vec, &a_mat, &b_vec, &h_mat, &g_vec);

            // Display as CSV
            print_results(n_iters, &x_vec);

            sweep_val += dc_params.step;
        }
    }
}

fn find_nodes(elems: &Vec<device::SpiceElem>) -> BTreeMap<String, device::RowType> {
    let mut nodes: BTreeMap<String, device::RowType> = BTreeMap::new();

    for elem in elems.iter() {
        for node in elem.nodes.iter() {
            nodes.insert(node.to_string(), device::RowType::Voltage);
        }

        if let device::DType::Vdd = elem.dtype {
            nodes.insert(elem.name.to_string(), device::RowType::Current);
        }
    }

    if !nodes.contains_key(GND) {
        panic!("GND not found!");
    }
    nodes.remove(GND);

    nodes
}

fn print_headers(nodes: &BTreeMap<String, device::RowType>) {
    println!(
        "n_iters{}",
        nodes.keys().fold(String::new(), |a, b| a + "," + b)
    );
}

fn print_results(n_iters: i64, x_vec: &Vec<f64>) {
    println!(
        "{}{}",
        n_iters,
        x_vec
            .iter()
            .map(|x| x.to_string())
            .fold(String::new(), |a, b| a + "," + &b)
    );
}
