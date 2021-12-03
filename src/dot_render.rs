use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::node::{LinkType, Node};

pub fn render(nodes: &[Node], path: impl AsRef<Path>) -> Result<()> {
    let mut f = OpenOptions::new().create(true).write(true).open(path)?;
    write!(f, "digraph{{")?;
    for (i, node) in nodes.iter().enumerate() {
        write!(
            f,
            "n{}[label=\"{}: {:?}\\n({}, {}, {})\"];",
            i, i, node.ty, node.pos.x, node.pos.y, node.pos.z
        )?;
        for link in &node.inputs {
            let color = match link.ty {
                LinkType::Default => "",
                LinkType::Side => ",color=\"blue\"",
            };
            write!(
                f,
                "n{}->n{}[label=\"{}\"{}];",
                link.to, i, link.weight, color
            )?;
        }
    }
    write!(f, "}}")?;

    Ok(())
}
