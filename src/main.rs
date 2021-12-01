mod node;

use node::Node;
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    let bytes = fs::read("chungus.bc")?;
    let nodes: Vec<Node> = bincode::deserialize(&bytes)?;
    dbg!(nodes.len());

    Ok(())
}
