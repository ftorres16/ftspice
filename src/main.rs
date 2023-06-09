use std::env;

extern crate ndarray;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use crate::engine::error::NotConvergedError;

mod command;
mod device;
mod engine;
mod node;
mod node_collection;
mod parser;
mod spice_fn;

fn main() -> Result<(), NotConvergedError> {
    let args: Vec<String> = env::args().collect();

    let file = &args
        .get(1)
        .expect("Insufficient arguments. Specify spice file to simulate.");

    let (elems, cmds) = parser::parse_spice_file(file);

    parser::check_elems::check_elems(&elems);

    let mut engine = engine::Engine::new(elems, cmds);

    if let Some(_) = engine.op_cmd {
        let res = engine.run_op()?;
        res.print();
    }

    if let Some(_) = engine.dc_cmd {
        let res = engine.run_dc()?;
        res.print();
    }

    if let Some(_) = engine.tran_cmd {
        let res = engine.run_tran()?;
        res.print();
    }

    Ok(())
}
