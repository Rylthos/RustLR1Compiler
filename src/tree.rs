use crate::lexer::TreeNode;

pub fn print_nodes(tree: &TreeNode, previous: &mut Vec<String>, first_node: bool, last_node: bool) {
    const ADDITIVE_SPACING: usize = 3;
    let symbol = if first_node && last_node {
        "═"
    } else if first_node {
        "╦"
    } else if last_node {
        "╚"
    } else {
        "╠"
    };

    match tree {
        TreeNode::Node(nodes) => {
            for (i, node) in (0usize..).zip(nodes.iter()) {
                let first = i == 0;
                let last = i == nodes.len() - 1;

                if let Some(p) = previous.last() {
                    if p != " " && p != "║" && !first {
                        if let Some(x) = previous.last_mut() {
                            *x = " ".to_string();
                        }
                    }
                }

                if first {
                    previous.push(format!("{symbol}"));
                } else if !last_node {
                    previous.push(format!("║"));
                } else {
                    previous.push(format!(" "));
                }

                print_nodes(node, previous, first, last);
                previous.pop();
            }
        }
        TreeNode::Leaf(s) => {
            let prev_string = previous.join("");

            println!("{prev_string}{symbol} {s}: \t{previous:?}");
        }
    }
}

pub fn print_tree(tree: &TreeNode) {
    println!("{tree:?}");
    let mut previous_string = vec![];
    print_nodes(tree, &mut previous_string, true, true);
}
