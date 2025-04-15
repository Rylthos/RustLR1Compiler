use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const EOS_T: &str = "EOS";
const START_T: &str = "^";
const EPS_T: &str = "EPS";

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct Rule {
    head: Type,
    tokens: Vec<Type>,
    dot: usize,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Ord, PartialOrd)]
enum Type {
    Terminal(String),
    NonTerminal(String),
}

fn create_rule(head: &str, rule: &str, terminals: &BTreeSet<String>) -> Rule {
    Rule {
        head: Type::NonTerminal(head.to_string()),
        tokens: rule
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| String::from(s.trim()))
            .map(|s| {
                if terminals.contains(&s) {
                    Type::Terminal(s)
                } else {
                    Type::NonTerminal(s)
                }
            })
            .collect::<Vec<_>>(),
        dot: 0,
    }
}

fn parse_config_file(file_name: &str) -> BTreeSet<Rule> {
    let contents = fs::read_to_string(file_name).expect("Failed to read file {file_name}");

    let rules = contents
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.split("->").collect::<Vec<_>>())
        .map(|v| {
            let rule = v[0];
            let remaining = v[1];
            (rule, remaining)
        })
        .map(|(name, rule)| {
            let rules = rule.split("|").map(|r| r.trim()).collect::<Vec<_>>();
            (name.trim(), rules)
        })
        .flat_map(|(name, rules)| {
            rules
                .iter()
                .map(|v| (name, v.to_string()))
                .collect::<Vec<_>>()
        });

    let terminals = rules.clone().map(|(name, _)| name.to_string()).collect();

    rules
        .map(|(name, rule)| create_rule(name, &rule, &terminals))
        .collect::<BTreeSet<_>>()
}

fn advance_dot(rule: &Rule) -> Rule {
    let new_dot = usize::min(rule.dot + 1, rule.tokens.len());
    Rule {
        head: rule.head.clone(),
        tokens: rule.tokens.clone(),
        dot: new_dot,
    }
}

fn after_dot(rule: &Rule) -> Type {
    if rule.dot == rule.tokens.len() {
        return Type::Terminal(EOS_T.to_string());
    }

    rule.tokens.get(rule.dot).unwrap().clone()
}

fn closure(rules: &BTreeSet<Rule>, all_rules: &BTreeSet<Rule>) -> BTreeSet<Rule> {
    let mut new_rules = BTreeSet::new();

    let mut stack: Vec<Rule> = Vec::new();
    for rule in rules {
        stack.push(rule.clone());
    }

    while !stack.is_empty() {
        let rule = stack.pop().unwrap();
        if new_rules.contains(&rule) {
            continue;
        };
        new_rules.insert(rule.clone());

        let current = after_dot(&rule);
        if let Type::NonTerminal(_) = current {
            for other_rule in all_rules {
                if other_rule.head == current {
                    stack.push(other_rule.clone());
                }
            }
        }
    }

    new_rules
}

fn advance_by(rules: &BTreeSet<Rule>, sym: &Type, all_rules: &BTreeSet<Rule>) -> BTreeSet<Rule> {
    let modified_rules = rules
        .iter()
        .filter(|r| {
            let next_sym = after_dot(r);
            match (next_sym, sym) {
                (Type::Terminal(s), Type::Terminal(s2))
                | (Type::NonTerminal(s), Type::NonTerminal(s2)) => s == *s2,
                _ => false,
            }
        })
        .map(|r| advance_dot(r))
        .collect();

    closure(&modified_rules, all_rules)
}

fn generate(
    rules: &BTreeSet<Rule>,
) -> (
    BTreeMap<usize, BTreeMap<Type, BTreeSet<Rule>>>,
    BTreeMap<(usize, Type), usize>,
) {
    let mut tables: BTreeMap<usize, BTreeMap<Type, BTreeSet<Rule>>> = BTreeMap::new();
    let mut lookup: BTreeMap<(usize, Type), usize> = BTreeMap::new();
    let mut lookup_set: BTreeMap<BTreeSet<Rule>, usize> = BTreeMap::new();

    tables.insert(
        0,
        vec![(Type::Terminal(START_T.to_string()), rules.clone())]
            .into_iter()
            .collect(),
    );
    lookup.insert((0, Type::Terminal(START_T.to_string())), 0);
    lookup_set.insert(rules.clone(), 0);

    let mut stack = Vec::new();
    stack.push((0usize, Type::Terminal(START_T.to_string())));

    let mut current_index = 1usize;

    while !stack.is_empty() {
        let (parent_index, move_sym) = stack.pop().unwrap();

        let index = lookup
            .get(&(parent_index, move_sym.clone()))
            .unwrap()
            .clone();
        let current_entries = tables.get(&parent_index).unwrap().get(&move_sym).unwrap();

        let advance_symbols: BTreeSet<Type> = current_entries
            .iter()
            .map(|r| after_dot(r))
            .filter(|r| *r != Type::Terminal(EOS_T.to_string()))
            .collect();

        let advanced = advance_symbols
            .iter()
            .map(|s| (s.clone(), advance_by(current_entries, s, rules)))
            .collect::<Vec<_>>();

        for (symbol, new_set) in advanced {
            if lookup_set.contains_key(&new_set) {
                let prev_index = *lookup_set.get(&new_set).unwrap();
                lookup.insert((index, symbol.clone()), prev_index);
                println!("{index} -> {prev_index}: {symbol:?} -> {new_set:?}");
            } else {
                tables.insert(current_index, BTreeMap::new());
                tables
                    .get_mut(&parent_index)
                    .unwrap()
                    .insert(symbol.clone(), new_set.clone());
                lookup_set.insert(new_set.clone(), current_index);
                lookup.insert((parent_index, symbol.clone()), current_index);
                stack.push((parent_index, symbol.clone()));

                println!("{index} -> {current_index}: {symbol:?} -> {new_set:?}");

                current_index += 1;
            }
        }
    }

    (tables, lookup)
}

fn nullable(all_rules: BTreeSet<Rule>) -> BTreeSet<Type> {
    let mut nullable_sym = BTreeSet::new();
    nullable_sym.insert(Type::Terminal(EPS_T.to_string()));

    let mut change = true;
    while change {
        change = false;

        for rule in all_rules.iter() {
            let mut nullable = true;
            for token in rule.tokens.iter() {
                if !nullable_sym.contains(&token) {
                    nullable = false;
                    break;
                }
            }

            if nullable {
                change = true;
                nullable_sym.insert(rule.head.clone());
            }
        }
    }

    nullable_sym
}

// fn first(all_rules: BTreeSet<Rule>) -> BTreeMap<Type, Type> {
//
// }

// fn follow(all_rules: BTreeSet<Rule>) -> BTreeMap<Type, Type> {
//
// }

pub fn generate_tables(file_name: &str) {
    let rules = parse_config_file(file_name);
    // let symbols = find_symbols(&rules);

    let (table, lookup) = generate(&rules);
    // let tables = generate(&rules, &symbols);
}
