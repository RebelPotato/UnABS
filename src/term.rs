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

// #[derive(Debug, Clone)]
// pub struct CompiledTerm(Vec<Instr>);

// #[derive(Debug, Clone)]
// pub struct Instr(Option<Primitive>); // lack of a primitive means push

// #[derive(Debug, Clone)]
// pub enum Primitive {
//   I, K, S, V, D, C, R, Put(char),
// }
// // impl Display for CompiledTerm {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         let reversed = self.0.iter().rev();
// //         for instr in reversed {
// //             match instr {
// //                 Instr(Some(Primitive::I)) => write!(f, "i")?,
// //                 Instr(Some(Primitive::K)) => write!(f, "k")?,
// //                 Instr(Some(Primitive::S)) => write!(f, "s")?,
// //                 Instr(Some(Primitive::V)) => write!(f, "v")?,
// //                 Instr(Some(Primitive::D)) => write!(f, "d")?,
// //                 Instr(Some(Primitive::C)) => write!(f, "c")?,
// //                 Instr(Some(Primitive::R)) => write!(f, "r")?,
// //                 Instr(Some(Primitive::Put(c))) => write!(f, ".{}", c)?,
// //                 Instr(None) => write!(f, "`")?,
// //             }
// //         }
// //         Ok(())
// //     }
// // }

// fn parse_to_compiled_term(pair: pest::iterators::Pair<Rule>, arr: &mut Vec<Instr>) {
//     match pair.as_rule() {
//         Rule::term => parse_to_compiled_term(pair.into_inner().next().unwrap(), arr),
//         Rule::atomic => match pair.as_str() {
//             "i" => arr.push(Instr(Some(Primitive::I))),
//             "s" => arr.push(Instr(Some(Primitive::S))),
//             "k" => arr.push(Instr(Some(Primitive::K))),
//             "v" => arr.push(Instr(Some(Primitive::V))),
//             "d" => arr.push(Instr(Some(Primitive::D))),
//             "c" => arr.push(Instr(Some(Primitive::C))),
//             "r" => arr.push(Instr(Some(Primitive::R))),
//             _ => unreachable!(),
//         },
//         Rule::putchar => {
//             let str = pair.as_str();
//             let char = str.chars().nth(1).unwrap();
//             arr.push(Instr(Some(Primitive::Put(char))));
//         }
//         Rule::app => {
//             let mut pairs = pair.into_inner();
//             parse_to_compiled_term(pairs.next().unwrap(), arr);
//             parse_to_compiled_term(pairs.next().unwrap(), arr);
//             arr.push(Instr(None));
//         }
//         _ => unreachable!(),
//     }
// }

// pub fn parse_compiled_term(s: &str) -> CompiledTerm {
//     let parsed = UnParser::parse(Rule::main, s)
//         .expect("parse error")
//         .next()
//         .unwrap();
//     // the term is the second child of the main rule
//     let mut pair = parsed.into_inner();
//     let term = pair.next().unwrap();
//     let mut comp = CompiledTerm(Vec::new());
//     parse_to_compiled_term(term, &mut comp.0);
//     comp.0.reverse();
//     comp
// }