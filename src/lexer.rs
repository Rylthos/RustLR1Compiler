use std::collections::BTreeMap;
use std::collections::VecDeque;

use crate::lrgenerator::{Action, Rule, Type, EOS_T, START_NT};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Token {
    Token(String),
}

#[derive(Debug, Clone)]
pub enum TreeNode {
    Node(Vec<TreeNode>),
    Leaf(String),
}

fn lex(input: &String) -> Vec<Token> {
    input
        .split(" ")
        .flat_map(|s| s.split("").filter(|s| !s.is_empty()))
        .map(|s| Token::Token(s.to_string()))
        .collect::<Vec<_>>()
}

fn reverse_tree(tree: &TreeNode) -> TreeNode {
    match tree {
        TreeNode::Leaf(_) => tree.clone(),
        TreeNode::Node(nodes) => {
            let mut new_nodes = Vec::new();
            new_nodes.reserve(nodes.len());
            for node in nodes {
                new_nodes.insert(0, reverse_tree(node));
            }

            TreeNode::Node(new_nodes)
        }
    }
}

pub fn parse_string(
    input: &String,
    parse_table: &BTreeMap<(usize, Type), Action>,
    reductions: &BTreeMap<usize, Rule>,
) {
    let tokens = lex(input);

    let mut state_stack: VecDeque<Type> = VecDeque::new();
    let mut stack: VecDeque<usize> = VecDeque::new();

    let mut tree: Option<TreeNode> = None;

    state_stack.push_back(Type::NonTerminal(START_NT.to_string()));
    stack.push_back(0usize);

    let mut index = 0usize;

    println!("{tokens:?}");

    loop {
        let item = if index == tokens.len() {
            EOS_T.to_string()
        } else {
            match tokens.get(index).unwrap().clone() {
                Token::Token(s) => s,
            }
        };

        if let Some(action) =
            parse_table.get(&(*stack.back().unwrap(), Type::Terminal(item.clone())))
        {
            match action {
                Action::Shift(next_index) => {
                    state_stack.push_back(Type::Terminal(item.clone()));
                    stack.push_back(*next_index);
                    index += 1;

                    if let Some(t) = tree.clone() {
                        match t {
                            TreeNode::Leaf(_) => {
                                tree = Some(TreeNode::Node(vec![t, TreeNode::Leaf(item.clone())]));
                            }
                            TreeNode::Node(n) => {
                                let mut new_vec = n.clone();
                                new_vec.push(TreeNode::Leaf(item.clone()));
                                tree = Some(TreeNode::Node(new_vec));
                            }
                        }
                    } else {
                        tree = Some(TreeNode::Leaf(item.clone()));
                    }
                }
                Action::Reduce(reduction) => {
                    let rule = reductions.get(reduction).unwrap();

                    println!("Reduce: {rule}");

                    let mut nodes: Vec<TreeNode> = Vec::new();

                    for _ in rule.tokens.iter() {
                        state_stack.pop_back();
                        stack.pop_back();

                        match tree.as_ref().unwrap() {
                            TreeNode::Leaf(_) => nodes.push(tree.clone().unwrap()),
                            TreeNode::Node(ns) => {
                                let mut ns = ns.clone();
                                nodes.push(ns.last().unwrap().clone());
                                ns.pop();
                                tree = Some(TreeNode::Node(ns));
                            }
                        }
                    }
                    state_stack.push_back(rule.head.clone());

                    match tree.as_ref().unwrap() {
                        TreeNode::Leaf(_) => tree = Some(TreeNode::Node(nodes)),
                        TreeNode::Node(ns) => {
                            let mut ns = ns.clone();
                            if nodes.len() == 1 {
                                ns.push(nodes.get(0).unwrap().clone());
                            } else {
                                ns.push(TreeNode::Node(nodes));
                            }
                            tree = Some(TreeNode::Node(ns))
                        }
                    }

                    // Get top of stack for Goto action
                    let next_action = parse_table
                        .get(&(*stack.back().unwrap(), state_stack.back().unwrap().clone()));
                    if let Some(Action::Goto(next_index)) = next_action {
                        stack.push_back(*next_index);
                    }
                }
                Action::Accept => break,
                _ => { /* Goto not a possible action */ }
            }
        } else {
            panic!(
                "Failed to parse: No action for {item} in {}",
                stack.back().unwrap()
            );
        }

        if index > tokens.len() {
            panic!("Parsing did not finish");
        }
    }

    let tree = reverse_tree(&tree.unwrap());

    println!("Finished parsing");
    println!("{tree:?}");
}
