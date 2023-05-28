pub const GND: &str = "0";

#[derive(Debug)]
pub struct MNANode {
    pub ntype: NodeType,
    pub idx: usize,
}

#[derive(Debug)]
pub enum NodeType {
    Voltage,
    Current,
}
