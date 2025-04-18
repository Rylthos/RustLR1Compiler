use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const EOS_T: &str = "$";
const START_T: &str = "START";
const START_NT: &str = "START";
const EPS_T: &str = "EPS";

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Rule {
    head: Type,
    tokens: Vec<Type>,
    dot: usize,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let body = self
            .tokens
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, t)| {
                if self.dot == i {
                    acc.push_str("•");
                }
                if i < self.tokens.len() {
                    acc.push_str(&format!("{t} "));
                } else {
                    acc.push_str(&format!("{t}"));
                }
                acc
            });

        write!(f, "{} -> {}", self.head, body)
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Ord, PartialOrd)]
pub enum Type {
    Terminal(String),
    NonTerminal(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Terminal(s) | Type::NonTerminal(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    Goto(usize),
    Accept,
}

fn create_rule(head: &str, rule: &str, non_terminals: &BTreeSet<String>) -> Rule {
    Rule {
        head: Type::NonTerminal(head.to_string()),
        tokens: rule
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| String::from(s.trim()))
            .map(|s| {
                if non_terminals.contains(&s) {
                    Type::NonTerminal(s)
                } else {
                    Type::Terminal(s)
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

    let non_terminals = rules.clone().map(|(name, _)| name.to_string()).collect();

    rules
        .map(|(name, rule)| create_rule(name, &rule, &non_terminals))
        .collect::<BTreeSet<_>>()
}

fn find_symbols(all_rules: &BTreeSet<Rule>) -> BTreeSet<Type> {
    let mut symbols = BTreeSet::new();
    for rule in all_rules {
        symbols.insert(rule.head.clone());

        for token in rule.tokens.iter() {
            symbols.insert(token.clone());
        }
    }

    symbols
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

fn generate_sets(
    rules: &BTreeSet<Rule>,
) -> (
    BTreeMap<usize, BTreeSet<Rule>>,
    BTreeMap<(usize, Type), usize>,
) {
    let mut table: BTreeMap<usize, BTreeSet<Rule>> = BTreeMap::new();
    let mut transitions: BTreeMap<(usize, Type), usize> = BTreeMap::new();
    let mut seen: BTreeMap<BTreeSet<Rule>, usize> = BTreeMap::new();

    table.insert(0, rules.clone());
    transitions.insert((0, Type::Terminal(START_T.to_string())), 0);
    seen.insert(rules.clone(), 0);

    let mut stack = Vec::new();
    stack.push((0usize, Type::Terminal(START_T.to_string())));

    let mut current_index = 1usize;

    while !stack.is_empty() {
        let (index, move_sym) = stack.pop().unwrap();

        let set_index = transitions.get(&(index, move_sym.clone())).unwrap().clone();
        let current_set = table.get(&set_index).unwrap();

        let advance_symbols: BTreeSet<Type> = current_set
            .iter()
            .map(|r| after_dot(r))
            .filter(|r| *r != Type::Terminal(EOS_T.to_string()))
            .collect();

        let advanced = advance_symbols
            .iter()
            .map(|s| (s.clone(), advance_by(current_set, s, rules)))
            .collect::<Vec<_>>();

        for (advance_symbol, new_set) in advanced {
            if seen.contains_key(&new_set) {
                let prev_index = seen.get(&new_set).unwrap();
                transitions.insert((set_index, advance_symbol.clone()), *prev_index);
            } else {
                table.insert(current_index, new_set.clone());
                transitions.insert((set_index, advance_symbol.clone()), current_index);
                seen.insert(new_set.clone(), current_index);

                stack.push((set_index, advance_symbol.clone()));

                current_index += 1;
            }
        }
    }

    transitions.remove(&(0, Type::Terminal(START_T.to_string())));

    (table, transitions)
}

fn nullable(all_rules: &BTreeSet<Rule>) -> BTreeSet<Type> {
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
                if !nullable_sym.contains(&rule.head) {
                    change = true;
                    nullable_sym.insert(rule.head.clone());
                }
            }
        }
    }

    nullable_sym
}

fn first(all_rules: &BTreeSet<Rule>, symbols: &BTreeSet<Type>) -> BTreeMap<Type, BTreeSet<Type>> {
    let mut first = BTreeMap::new();

    for symbol in symbols {
        match symbol {
            Type::Terminal(s) => {
                first.insert(
                    Type::Terminal(s.clone()),
                    vec![Type::Terminal(s.clone())].into_iter().collect(),
                );
            }
            Type::NonTerminal(s) => {
                first.insert(Type::NonTerminal(s.clone()), BTreeSet::new());
            }
        }
    }

    let nullable_set = nullable(all_rules);

    let mut changed = true;
    while changed {
        changed = false;

        for rule in all_rules.into_iter() {
            if !first
                .get(&rule.head)
                .unwrap()
                .contains(&Type::Terminal(EPS_T.to_string()))
            {
                let mut isnullable = true;
                for token in rule.tokens.iter() {
                    if !nullable_set.contains(&token) {
                        isnullable = false;
                        break;
                    }
                }

                if isnullable {
                    first
                        .get_mut(&rule.head)
                        .unwrap()
                        .insert(Type::Terminal(EPS_T.to_string()));
                    changed = true;
                }
            }

            for token in rule.tokens.iter() {
                let new_set = first
                    .get(token)
                    .unwrap()
                    .difference(
                        &vec![Type::Terminal(EPS_T.to_string())]
                            .into_iter()
                            .collect(),
                    )
                    .cloned()
                    .collect::<BTreeSet<_>>();

                let current = first.get(&rule.head).unwrap();
                let union = current.union(&new_set).cloned().collect::<BTreeSet<_>>();

                if !union
                    .symmetric_difference(current)
                    .collect::<Vec<_>>()
                    .is_empty()
                {
                    *first.get_mut(&rule.head).unwrap() = union;
                    changed = true;
                }

                if !nullable_set.contains(token) {
                    break;
                }
            }
        }
    }

    first
}

fn follow(
    first: &BTreeMap<Type, BTreeSet<Type>>,
    all_rules: &BTreeSet<Rule>,
    symbols: &BTreeSet<Type>,
) -> BTreeMap<Type, BTreeSet<Type>> {
    let mut follow = BTreeMap::new();

    for symbol in symbols {
        match symbol {
            Type::Terminal(_) => {}
            Type::NonTerminal(s) => {
                follow.insert(Type::NonTerminal(s.clone()), BTreeSet::new());
            }
        }
    }

    follow.insert(
        Type::NonTerminal(START_NT.to_string()),
        vec![Type::Terminal(EOS_T.to_string())]
            .into_iter()
            .collect(),
    );

    let nullable = nullable(all_rules);

    let mut changed = true;
    while changed {
        changed = false;

        for rule in all_rules {
            let mut iter = rule.tokens.iter();

            let mut current = iter.next();
            let iter_copy = iter.clone();
            let mut next = iter.next();

            while let Some(token) = current {
                if let Type::NonTerminal(_) = token {
                    let current_set = follow.get(&token).unwrap();

                    if let Some(next_token) = next {
                        let mut new_set = BTreeSet::new();
                        if *next_token != Type::Terminal(EPS_T.to_string()) {
                            let first = first
                                .get(&next_token)
                                .unwrap()
                                .difference(
                                    &vec![Type::Terminal(EPS_T.to_string())]
                                        .into_iter()
                                        .collect(),
                                )
                                .cloned()
                                .collect::<BTreeSet<_>>();

                            new_set = current_set.union(&first).cloned().collect::<BTreeSet<_>>();
                            if !new_set
                                .symmetric_difference(&current_set)
                                .collect::<Vec<_>>()
                                .is_empty()
                            {
                                changed = true;
                            }
                        }

                        let mut remaining_are_nullable = true;
                        for next_tok in iter_copy.clone() {
                            if !nullable.contains(next_tok) {
                                remaining_are_nullable = false;
                                break;
                            }
                        }
                        if remaining_are_nullable {
                            let other = follow.get(&rule.head).unwrap().clone();
                            new_set = current_set
                                .union(&other)
                                .cloned()
                                .collect::<BTreeSet<_>>()
                                .union(&new_set)
                                .cloned()
                                .collect::<BTreeSet<_>>();

                            if !new_set
                                .symmetric_difference(&current_set)
                                .collect::<Vec<_>>()
                                .is_empty()
                            {
                                changed = true;
                            }
                        }

                        if !new_set.is_empty() {
                            *follow.get_mut(&token).unwrap() = new_set;
                        }
                    } else {
                        let head = follow.get(&rule.head).unwrap();
                        let new_set = current_set.union(&head).cloned().collect::<BTreeSet<_>>();

                        if !new_set
                            .symmetric_difference(&current_set)
                            .collect::<Vec<_>>()
                            .is_empty()
                        {
                            *follow.get_mut(&token).unwrap() = new_set;
                            changed = true;
                        }
                    }
                }

                current = next;
                next = iter.next()
            }
        }
    }

    follow
}

pub fn generate_table(file_name: &str) -> (BTreeMap<(usize, Type), Action>, BTreeMap<usize, Rule>) {
    let rules = parse_config_file(file_name);
    let symbols = find_symbols(&rules);

    let (sets, transitions) = generate_sets(&rules);

    let first = first(&rules, &symbols);
    let follow = follow(&first, &rules, &symbols);

    let reductions = rules.iter().zip(0usize..).collect::<BTreeMap<_, _>>();
    let reductions_rev = rules
        .clone()
        .into_iter()
        .zip(0usize..)
        .map(|(r, i)| (i, r))
        .collect::<BTreeMap<_, _>>();

    let mut table: BTreeMap<(usize, Type), Action> = BTreeMap::new();
    for ((index, symbol), next_index) in transitions.into_iter() {
        if let Type::Terminal(_) = symbol {
            table.insert((index, symbol), Action::Shift(next_index));
        } else {
            table.insert((index, symbol), Action::Goto(next_index));
        }
    }

    for (index, set) in sets {
        for rule in set {
            if after_dot(&rule) == Type::Terminal(EOS_T.to_string()) {
                if rule.head != Type::NonTerminal(START_NT.to_string()) {
                    let follow_set = follow.get(&rule.head).unwrap();
                    for f in follow_set {
                        let reduction = reductions
                            .get(&Rule {
                                head: rule.head.clone(),
                                tokens: rule.tokens.clone(),
                                dot: 0,
                            })
                            .unwrap();
                        if table.contains_key(&(index, f.clone())) {
                            panic!("Error: values shared: {index}: {f:?}");
                        }
                        table.insert((index, f.clone()), Action::Reduce(*reduction));
                    }
                } else {
                    table.insert((index, Type::Terminal(EOS_T.to_string())), Action::Accept);
                }
            }
        }
    }

    (table, reductions_rev)
}

pub fn print_table(table: &BTreeMap<(usize, Type), Action>, reductions: &BTreeMap<usize, Rule>) {
    println!("Reductions");
    for (index, set) in reductions {
        println!("{index}: {set}");
    }

    let states = table
        .iter()
        .map(|((index, _), _)| index)
        .collect::<BTreeSet<_>>();

    let min_size = usize::max(states.last().unwrap().to_string().len() + 1, 3);

    let get_rule_size = |r: &Type| match r {
        Type::Terminal(s) | Type::NonTerminal(s) => usize::max(s.len(), min_size),
    };

    let modify_sizes = |r: &Type, terminal: &mut usize, non_terminal: &mut usize| {
        let size = get_rule_size(r);
        match r {
            Type::Terminal(_) => *terminal += size + 3,
            Type::NonTerminal(_) => *non_terminal += size + 3,
        }
    };

    let mut padding: BTreeMap<Type, usize> = BTreeMap::new();
    let mut terminal_size: usize = 0;
    let mut non_terminal_size: usize = 0;

    for (_, rule) in reductions {
        if rule.head != Type::NonTerminal(START_NT.to_string()) {
            if !padding.contains_key(&rule.head) {
                modify_sizes(&rule.head, &mut terminal_size, &mut non_terminal_size);
            }

            let size = get_rule_size(&rule.head);
            padding.insert(rule.head.clone(), size);
        }

        for token in rule.tokens.iter() {
            if *token == Type::NonTerminal(START_NT.to_string()) {
                continue;
            }

            if !padding.contains_key(&token) {
                modify_sizes(&token, &mut terminal_size, &mut non_terminal_size);
            }

            padding.insert(token.clone(), get_rule_size(&token));
        }
    }
    padding.insert(
        Type::Terminal(EOS_T.to_string()),
        get_rule_size(&Type::Terminal(EOS_T.to_string())),
    );
    modify_sizes(
        &Type::Terminal(EOS_T.to_string()),
        &mut terminal_size,
        &mut non_terminal_size,
    );

    println!(
        "STATE |{:^twidth$}|{:^ntwidth$}|",
        "Action",
        "Goto",
        twidth = terminal_size - 1,
        ntwidth = non_terminal_size - 1,
    );
    let mut t = String::new();
    let mut nt = String::new();
    for (v, p) in padding.iter() {
        match v {
            Type::Terminal(s) => t.push_str(&format!(" {:^width$} |", s, width = p)),
            Type::NonTerminal(s) => nt.push_str(&format!(" {:^width$} |", s, width = p)),
        }
    }
    println!("      |{t}{nt}");

    t = String::new();
    nt = String::new();
    for (v, p) in padding.iter() {
        match v {
            Type::Terminal(_) => t.push_str(&format!("-{:-^width$}-+", "", width = p)),
            Type::NonTerminal(_) => nt.push_str(&format!("-{:-^width$}-+", "", width = p)),
        }
    }
    println!("------+{t}{nt}");
    for state in states {
        t = String::new();
        nt = String::new();
        for (v, p) in padding.iter() {
            match v {
                Type::Terminal(_) => {
                    let action = table.get(&(state.clone(), v.clone()));
                    if let Some(a) = action {
                        match a {
                            Action::Shift(i) => {
                                t.push_str(&format!(" {:>width$} |", format!("S{i}"), width = p))
                            }
                            Action::Reduce(i) => {
                                t.push_str(&format!(" {:>width$} |", format!("R{i}"), width = p))
                            }
                            Action::Accept => {
                                t.push_str(&format!(" {:>width$} |", "ACC", width = p))
                            }
                            Action::Goto(_) => t.push_str(&format!(" {:>width$} |", "", width = p)),
                        }
                    } else {
                        t.push_str(&format!(" {:>width$} |", "", width = p))
                    }
                }
                Type::NonTerminal(_) => {
                    let action = table.get(&(state.clone(), v.clone()));
                    if let Some(a) = action {
                        match a {
                            Action::Goto(i) => {
                                nt.push_str(&format!(" {:>width$} |", format!("G{i}"), width = p))
                            }
                            _ => nt.push_str(&format!(" {:>width$} |", "", width = p)),
                        }
                    } else {
                        nt.push_str(&format!(" {:>width$} |", "", width = p))
                    }
                }
            }
        }
        println!("{:^6}|{t}{nt}", state);
    }
}
