use std::collections::HashMap;

#[derive(Debug)]
pub enum CmdType {
    Op,
}

#[derive(Debug)]
pub struct Command {
    pub ctype: CmdType,
    pub params: HashMap<String, f64>,
}
