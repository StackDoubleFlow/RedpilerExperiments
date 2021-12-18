use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::node::{LinkType, Node, NodeType};

pub fn render(nodes: &[Node], path: impl AsRef<Path>) -> Result<()> {
    let mut f = OpenOptions::new().create(true).write(true).open(path)?;
    write!(f, "digraph{{")?;
    for (i, node) in nodes.iter().enumerate() {
        let name = if matches!(node.ty, NodeType::Constant) {
            format!("Constant({})", node.output_power)
        } else {
            format!("{:?}", node.ty)
        };
        write!(
            f,
            "n{}[label=\"{}: {}\\n({}, {}, {})\"];",
            i, i, name, node.pos.x, node.pos.y, node.pos.z
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
