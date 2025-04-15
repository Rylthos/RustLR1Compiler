use std::env;

mod lrgenerator;
use crate::lrgenerator::generate_tables;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Invalid number of arguments: lrparser <config_file>")
    }
    let file_name = args.get(1).unwrap();
    generate_tables(file_name);
}
