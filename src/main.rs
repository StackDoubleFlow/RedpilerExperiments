mod node;
mod group;
mod constant_fold;
mod dot_render;

use node::Node;
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    let bytes = fs::read("small.bc")?;
    let mut nodes: Vec<Node> = bincode::deserialize(&bytes)?;
    dbg!(nodes.len());

    constant_fold::constant_fold(&mut nodes);
    group::group(&nodes);

    dot_render::render(&nodes, "graph.dot")?;

    Ok(())
}
