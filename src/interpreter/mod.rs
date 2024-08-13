use pest::Parser;
use pest_derive::Parser;
use std::char;
use std::fmt::Display;
use std::io::Write;

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

#[derive(Debug, Clone)]
pub enum Value {
    I0,
    S0,
    K0,
    V0,
    D0,
    C0,
    Put0(char),
    S1(Box<Value>),
    S2(Box<Value>, Box<Value>),
    K1(Box<Value>),
    D1(Box<Term>),
    C1(Box<Option<Kont>>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::I0 => write!(f, "i"),
            Value::S0 => write!(f, "s"),
            Value::K0 => write!(f, "k"),
            Value::V0 => write!(f, "v"),
            Value::D0 => write!(f, "d"),
            Value::C0 => write!(f, "c"),
            Value::Put0(c) => {
                if *c == '\n' {
                    write!(f, "r")
                } else {
                    write!(f, ".{}", c)
                }
            }
            Value::S1(w) => write!(f, "`s{}", w),
            Value::S2(w0, w1) => write!(f, "``s{}{}", w0, w1),
            Value::K1(w) => write!(f, "`k{}", w),
            Value::D1(t) => write!(f, "`d[{}]", t),
            Value::C1(_) => write!(f, "`c[?]"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Kont {
    BindT(Box<Term>, Box<Option<Kont>>),
    BindV(Box<Value>, Box<Option<Kont>>),
    BindW(Box<Value>, Box<Option<Kont>>),
    S2(Box<Value>, Box<Value>, Box<Option<Kont>>),
    S1(Box<Value>, Box<Option<Kont>>),
}
impl Display for Kont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kont::BindT(t, k) => {
                write!(f, "BindT({})", t)?;
                if let Some(k) = k.as_ref() {
                    write!(f, " {}", k)
                } else {
                    write!(f, "")
                }
            }
            Kont::BindV(v, k) => {
                write!(f, "BindV({})", v)?;
                if let Some(k) = k.as_ref() {
                    write!(f, " {}", k)
                } else {
                    write!(f, "")
                }
            }
            Kont::BindW(w, k) => {
                write!(f, "BindW({})", w)?;
                if let Some(k) = k.as_ref() {
                    write!(f, " {}", k)
                } else {
                    write!(f, "")
                }
            }
            Kont::S2(w0, w1, k) => {
                write!(f, "S2({}; {})", w0, w1)?;
                if let Some(k) = k.as_ref() {
                    write!(f, " {}", k)
                } else {
                    write!(f, "")
                }
            }
            Kont::S1(w, k) => {
                write!(f, "S1({})", w)?;
                if let Some(k) = k.as_ref() {
                    write!(f, " {}", k)
                } else {
                    write!(f, "")
                }
            }
        }
    }
}

pub enum State {
    Eval(Term, Option<Kont>),
    ApplyT(Value, Term, Option<Kont>),
    ApplyV(Value, Value, Option<Kont>),
    ApplyKont(Option<Kont>, Value),
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Eval(t, k) => {
                write!(f, "State: Eval\nTerm: [{}]\n", t)?;
                match k {
                    None => write!(f, "Kont: ()"),
                    Some(k) => write!(f, "Kont: {}", k),
                }
            }
            State::ApplyT(v, t, k) => {
                write!(f, "State: ApplyT\nValue: {}\nTerm: [{}]\n", v, t)?;
                match k {
                    None => write!(f, "Kont: ()"),
                    Some(k) => write!(f, "Kont: {}", k),
                }
            }
            State::ApplyV(v, w, k) => {
                write!(f, "State: ApplyV\nValue: {}\nWalue: {}\n", v, w)?;
                match k {
                    None => write!(f, "Kont: ()"),
                    Some(k) => write!(f, "Kont: {}", k),
                }
            }
            State::ApplyKont(k, v) => {
                write!(f, "State: ApplyKont\nValue: {}\n", v)?;
                match k {
                    None => write!(f, "Kont: ()"),
                    Some(k) => write!(f, "Kont: {}", k),
                }
            }
        }
    }
}

fn eval(t: Term, k: Option<Kont>) -> State {
    match t {
        Term::I => State::ApplyKont(k, Value::I0),
        Term::S => State::ApplyKont(k, Value::S0),
        Term::K => State::ApplyKont(k, Value::K0),
        Term::V => State::ApplyKont(k, Value::V0),
        Term::D => State::ApplyKont(k, Value::D0),
        Term::C => State::ApplyKont(k, Value::C0),
        Term::R => State::ApplyKont(k, Value::Put0('\n')),
        Term::Put(c) => State::ApplyKont(k, Value::Put0(c)),
        Term::App(t0, t1) => State::Eval(*t0, Some(Kont::BindT(t1, Box::new(k)))),
    }
}

fn apply_t(v: Value, t: Term, k: Option<Kont>) -> State {
    match v {
        Value::D0 => State::ApplyKont(k, Value::D1(Box::new(t))),
        _ => State::Eval(t, Some(Kont::BindV(Box::new(v), Box::new(k)))),
    }
}

fn apply_v(v: Value, w: Value, k: Option<Kont>) -> State {
    match v {
        Value::I0 => State::ApplyKont(k, w),
        Value::Put0(c) => {
            print!("{}", c);
            std::io::stdout().flush().unwrap();
            State::ApplyKont(k, w)
        }
        Value::S0 => State::ApplyKont(k, Value::S1(Box::new(w))),
        Value::S1(w0) => State::ApplyKont(k, Value::S2(w0, Box::new(w))),
        Value::S2(w0, w1) => {
            State::ApplyV(*w0, w.clone(), Some(Kont::S2(w1, Box::new(w), Box::new(k))))
        }
        Value::K0 => State::ApplyKont(k, Value::K1(Box::new(w))),
        Value::K1(w0) => State::ApplyKont(k, *w0),
        Value::V0 => State::ApplyKont(k, Value::V0),
        Value::C0 => State::ApplyV(w, Value::C1(Box::new(k.clone())), k),
        Value::C1(k1) => State::ApplyKont(*k1, w),
        Value::D1(t0) => State::Eval(*t0, Some(Kont::BindW(Box::new(w), Box::new(k)))),
        Value::D0 => unreachable!(),
    }
}

fn apply_kont(k: Kont, w: Value) -> State {
    match k {
        Kont::BindT(t, k1) => State::ApplyT(w, *t, *k1),
        Kont::BindV(v, k1) => State::ApplyV(*v, w, *k1),
        Kont::BindW(w1, k1) => State::ApplyV(w, *w1, *k1),
        Kont::S2(w1, w2, k1) => State::ApplyV(*w1, *w2, Some(Kont::S1(Box::new(w), k1))),
        Kont::S1(l, k1) => State::ApplyV(*l, w, *k1),
    }
}

pub enum SEither {
    S(State),
    V(Value),
}

pub fn new(t: Term) -> State {
    State::Eval(t, None)
}

impl State {
    pub fn step(self) -> SEither {
        match self {
            State::Eval(t, k) => SEither::S(eval(t, k)),
            State::ApplyT(v, t, k) => SEither::S(apply_t(v, t, k)),
            State::ApplyV(v, w, k) => SEither::S(apply_v(v, w, k)),
            State::ApplyKont(Some(k), w) => SEither::S(apply_kont(k, w)),
            State::ApplyKont(None, v) => SEither::V(v),
        }
    }

    pub fn run(self) -> Value {
        let mut state = self;
        loop {
            match state.step() {
                SEither::S(s) => state = s,
                SEither::V(v) => return v,
            }
        }
    }
    // todo: add a repl
}
