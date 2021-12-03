mod codegen;
mod constant_fold;
mod dot_render;
mod group;
mod node;

use anyhow::Result;
use node::Node;
use std::fs;

fn main() -> Result<()> {
    let bytes = fs::read("small.bc")?;
    let mut nodes: Vec<Node> = bincode::deserialize(&bytes)?;
    dbg!(nodes.len());

    constant_fold::constant_fold(&mut nodes);
    group::group(&nodes);

    codegen::gen(&nodes)?;

    dot_render::render(&nodes, "graph.dot")?;

    Ok(())
}
