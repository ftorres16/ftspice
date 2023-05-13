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
        let (n_iters, nodes, mut x) = engine.run_op();

        let mut headers = nodes.keys().map(|s| s.as_str()).collect::<Vec<_>>();
        headers.insert(0, "n_iters");
        print_headers(&headers);

        x.insert(0, n_iters.clone() as f64);
        print_results(&x);
    }

    if let Some(_) = engine.dc_cmd {
        let (n_iters_hist, nodes, x_hist) = engine.run_dc();

        let mut headers = nodes.keys().map(|s| s.as_str()).collect::<Vec<_>>();
        headers.insert(0, "n_iters");
        print_headers(&headers);
        for (n_iters, x) in n_iters_hist.iter().zip(x_hist) {
            let mut x = x.clone();
            x.insert(0, n_iters.clone() as f64);
            print_results(&x);
        }
    }

    if let Some(_) = engine.tran_cmd {
        let (n_iters_hist, nodes, t_hist, x_hist) = engine.run_tran();

        let mut headers = nodes.keys().map(|s| s.as_str()).collect::<Vec<_>>();
        headers.insert(0, "n_iters");
        headers.insert(1, "t");
        print_headers(&headers);

        for ((n_iters, t), x) in n_iters_hist.iter().zip(t_hist.iter()).zip(x_hist) {
            let mut x = x.clone();
            x.insert(0, n_iters.clone() as f64);
            x.insert(1, t.clone());
            print_results(&x);
        }
    }
}

fn print_headers(row: &Vec<&str>) {
    println!("{}", row.join(","));
}

fn print_results(row: &Vec<f64>) {
    println!(
        "{}",
        row.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
}
