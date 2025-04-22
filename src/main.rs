use std::env;

mod lexer;
mod lrgenerator;
mod tree;
use crate::lexer::parse_string;
use crate::lrgenerator::{generate_table, print_table};
use crate::tree::print_tree;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Invalid number of arguments: lrparser <config_file> <input_string>")
    }
    let file_name = args.get(1).unwrap();
    let (table, reductions) = generate_table(file_name);

    print_table(&table, &reductions);

    let tree = parse_string(args.get(2).unwrap(), &table, &reductions);

    print_tree(&tree);
}
