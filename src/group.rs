use crate::node::{Node, NodeId, NodeType};

fn walk_back(nodes: &[Node], start: NodeId) -> NodeId {
    let node = &nodes[start];
    if node.inputs.len() == 1 && nodes[node.inputs[0].to].updates.len() == 1 {
        walk_back(nodes, node.inputs[0].to)
    } else {
        start
    }
}

pub fn group(nodes: &[Node]) {
    let mut groups = vec![None; nodes.len()];
    let mut num_groups = 0;
    for (i, node) in nodes.iter().enumerate() {
        if node.ty == NodeType::Wire || groups[i].is_some() {
            continue;
        }

        let start = walk_back(nodes, i);
        if i == start {
            // No grouping necessary
            continue;
        }

        let group = num_groups;
        num_groups += 1;

        let mut ptr = start;
        loop {
            groups[ptr] = Some(group);
            if nodes[ptr].updates.len() == 1 {
                ptr = nodes[ptr].updates[0];
            } else {
                break;
            }
        }
    }

    let num_grouped = groups.iter().filter(|x| x.is_some()).count();
    dbg!(groups);
    dbg!(num_grouped);
}
