use crate::node::{Node, NodeType};

pub fn constant_fold(nodes: &mut [Node]) {
    let mut foldable = 0;
    loop {
        let old_foldable = foldable;
        'nodes: for i in 0..nodes.len() {
            let node = &nodes[i];
            if matches!(
                node.ty,
                NodeType::Constant
                    | NodeType::Lever
                    | NodeType::StonePressurePlate
                    | NodeType::StoneButton
            ) {
                continue;
            }
            for input in &node.inputs {
                if nodes[input.to].ty != NodeType::Constant {
                    continue 'nodes;
                }
            }

            foldable += 1;

            nodes[i].ty = NodeType::Constant;
            for input in nodes[i].inputs.clone() {
                nodes[input.to].updates.clear();
            }
            nodes[i].inputs.clear();
        }
        if foldable == old_foldable {
            break;
        }
    }
}
