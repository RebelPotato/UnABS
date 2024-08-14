use pest::Parser;
use pest_derive::Parser;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Term {
    I,
    S,
    K,
    V,
    D,
    C,
    R,
    Put(char),
    App(Box<Term>, Box<Term>),
    // Can do better: a flat AST
}

#[derive(Parser)]
#[grammar = "unlambda.pest"]
struct UnParser;

fn parse_to_term(pair: pest::iterators::Pair<Rule>) -> Term {
    match pair.as_rule() {
        Rule::term => parse_to_term(pair.into_inner().next().unwrap()),
        Rule::atomic => match pair.as_str() {
            "i" => Term::I,
            "s" => Term::S,
            "k" => Term::K,
            "v" => Term::V,
            "d" => Term::D,
            "c" => Term::C,
            "r" => Term::R,
            _ => unreachable!(),
        },
        Rule::putchar => {
            let str = pair.as_str();
            let char = str.chars().nth(1).unwrap();
            Term::Put(char)
        }
        Rule::app => {
            let mut pairs = pair.into_inner();
            let t0 = parse_to_term(pairs.next().unwrap());
            let t1 = parse_to_term(pairs.next().unwrap());
            Term::App(Box::new(t0), Box::new(t1))
        }
        _ => unreachable!(),
    }
}

pub fn parse_term(s: &str) -> Term {
    let parsed = UnParser::parse(Rule::main, s)
        .expect("parse error")
        .next()
        .unwrap();
    // the term is the second child of the main rule
    let mut pair = parsed.into_inner();
    let term = pair.next().unwrap();
    parse_to_term(term)
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::I => write!(f, "i"),
            Term::S => write!(f, "s"),
            Term::K => write!(f, "k"),
            Term::V => write!(f, "v"),
            Term::D => write!(f, "d"),
            Term::C => write!(f, "c"),
            Term::R => write!(f, "r"),
            Term::Put(c) => {
                if *c == '\n' {
                    write!(f, "r")
                } else {
                    write!(f, ".{}", c)
                }
            }
            Term::App(t0, t1) => write!(f, "`{}{}", t0, t1),
        }
    }
}