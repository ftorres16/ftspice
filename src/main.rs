use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod command;
mod device;
mod engine;
mod node;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args
        .get(1)
        .expect("Insufficient arguments. Specify spice file to simulate.");

    let (elems, cmds) = parser::parse_spice_file(file);

    let mut engine = engine::Engine::new(elems, cmds);

    if let Some(_) = engine.op_cmd {
        let (n_iters, x) = engine.run_op();

        print_headers(&engine.nodes);
        print_results(&n_iters, &x)
    }

    if let Some(_) = engine.dc_cmd {
        let (n_iters_hist, x_hist) = engine.run_dc();

        print_headers(&engine.nodes);
        for (n_iters, x) in n_iters_hist.iter().zip(x_hist) {
            print_results(n_iters, &x);
        }
    }

    if let Some(_) = engine.tran_cmd {
        let (n_iters_hist, t_hist, x_hist) = engine.run_tran();

        print_headers_tran(&engine.nodes);
        for ((n_iters, t), x) in n_iters_hist.iter().zip(t_hist.iter()).zip(x_hist) {
            print_results_tran(n_iters, t, &x);
        }
    }
}

fn print_headers(nodes: &node::NodeCollection) {
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

fn print_headers_tran(nodes: &node::NodeCollection) {
    println!(
        "n_iters,t{}",
        nodes.keys().fold(String::new(), |a, b| a + "," + b)
    );
}

fn print_results_tran(n_iters: &u64, t: &f64, x_vec: &Vec<f64>) {
    println!(
        "{},{}{}",
        n_iters,
        t,
        x_vec
            .iter()
            .map(|x| x.to_string())
            .fold(String::new(), |a, b| a + "," + &b)
    );
}
