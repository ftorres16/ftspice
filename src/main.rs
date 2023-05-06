use std::collections::BTreeMap;
use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod command;
mod device;
mod engine;
mod linear_stamp;
mod nonlinear_func;
mod nonlinear_stamp;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args
        .get(1)
        .expect("Insufficient arguments. Specify spice file to simulate.");

    let (elems, cmds) = parser::parse_spice_file(file);

    let engine = engine::Engine::new(elems, cmds);

    print_headers(&engine.nodes);

    if let Some(_) = engine.op_cmd {
        let (n_iters, x) = engine.run_op();

        if let None = engine.dc_cmd {
            print_results(&n_iters, &x)
        }
    }

    if let Some(_) = engine.dc_cmd {
        let (n_iters_hist, x_hist) = engine.run_dc();

        for (n_iters, x) in n_iters_hist.iter().zip(x_hist) {
            print_results(n_iters, &x);
        }
    }
}

fn print_headers(nodes: &BTreeMap<String, device::RowType>) {
    println!(
        "n_iters{}",
        nodes.keys().fold(String::new(), |a, b| a + "," + b)
    );
}

fn print_results(n_iters: &u64, x_vec: &Vec<f64>) {
    println!(
        "{}{}",
        n_iters,
        x_vec
            .iter()
            .map(|x| x.to_string())
            .fold(String::new(), |a, b| a + "," + &b)
    );
}
