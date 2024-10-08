use std::char;
use std::fmt::Display;
use std::io::Write;
use crate::term::Term;

// A copying abstract machine for unlambda
// Not the most efficient implementation!

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
    D1T(Box<Term>),
    D1V(Box<Value>),
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
            Value::D1T(t) => write!(f, "`d[{}]", t),
            Value::D1V(t) => write!(f, "`d{}", t),
            Value::C1(k) => match k.as_ref() {
                Some(k) => write!(f, "`c({})", k),
                None => write!(f, "`c()"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Kont {
    BindT(Box<Term>, Box<Option<Kont>>),
    BindV(Box<Value>, Box<Option<Kont>>),
    BindW(Box<Value>, Box<Option<Kont>>),
    SWait(Box<Value>, Box<Value>, Box<Option<Kont>>),
}
impl Display for Kont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut wrapped = "()".to_string();
        let mut current = self;
        loop {
            let (next, text) = match current {
                Kont::BindT(t, k) => (k, format!("`{}[{}]", wrapped, t)),
                Kont::BindV(v, k) => (k, format!("`{}{}", v, wrapped)),
                Kont::BindW(w, k) => (k, format!("`{}{}", wrapped, w)),
                Kont::SWait(v1, v, k) => (k, format!("`{}`{}{}", wrapped, v1, v)),
            };
            match next.as_ref() {
                Some(k) => {
                    wrapped = text;
                    current = k;
                }
                None => {
                    return write!(f, "{}", text);
                }
            }
        }
    }
}

pub enum State {
    Eval(Term, Option<Kont>),
    ApplyT(Value, Term, Option<Kont>),
    ApplyV(Value, Value, Option<Kont>),
    ApplyK(Option<Kont>, Value),
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
            State::ApplyK(k, v) => {
                write!(f, "State: ApplyK\nValue: {}\n", v)?;
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
        Term::I => State::ApplyK(k, Value::I0),
        Term::S => State::ApplyK(k, Value::S0),
        Term::K => State::ApplyK(k, Value::K0),
        Term::V => State::ApplyK(k, Value::V0),
        Term::D => State::ApplyK(k, Value::D0),
        Term::C => State::ApplyK(k, Value::C0),
        Term::R => State::ApplyK(k, Value::Put0('\n')),
        Term::Put(c) => State::ApplyK(k, Value::Put0(c)),
        Term::App(t0, t1) => State::Eval(*t0, Some(Kont::BindT(t1, Box::new(k)))),
    }
}

fn apply_t(v: Value, t: Term, k: Option<Kont>) -> State {
    match v {
        Value::D0 => State::ApplyK(k, Value::D1T(Box::new(t))),
        _ => State::Eval(t, Some(Kont::BindV(Box::new(v), Box::new(k)))),
    }
}

fn apply_v(v: Value, w: Value, k: Option<Kont>) -> State {
    match v {
        Value::I0 => State::ApplyK(k, w),
        Value::Put0(c) => {
            print!("{}", c);
            std::io::stdout().flush().unwrap();
            State::ApplyK(k, w)
        }
        Value::K0 => State::ApplyK(k, Value::K1(Box::new(w))),
        Value::K1(w0) => State::ApplyK(k, *w0),
        Value::V0 => State::ApplyK(k, Value::V0),
        // This clones the kontinuation. How can we avoid this?
        Value::C0 => State::ApplyV(w, Value::C1(Box::new(k.clone())), k),
        Value::C1(k1) => State::ApplyK(*k1, w),
        Value::D0 => State::ApplyK(k, Value::D1V(Box::new(w))),
        Value::D1T(t0) => State::Eval(*t0, Some(Kont::BindW(Box::new(w), Box::new(k)))),
        Value::D1V(v0) => State::ApplyV(*v0, w, k),
        Value::S0 => State::ApplyK(k, Value::S1(Box::new(w))),
        Value::S1(v0) => State::ApplyK(k, Value::S2(v0, Box::new(w))),
        Value::S2(v0, v1) => {
            // This copys the third value. A tree clone! Very inefficient.
            // How do we share? Rc? Cow? Make a flat list or something?
            State::ApplyV(*v0, w.clone(), Some(Kont::SWait(v1, Box::new(w), Box::new(k))))
        }
    }
}

fn apply_k(k: Kont, w: Value) -> State {
    match k {
        Kont::BindT(t, k) => State::ApplyT(w, *t, *k),
        Kont::BindV(v, k) => State::ApplyV(*v, w, *k),
        Kont::BindW(w1, k) => State::ApplyV(w, *w1, *k),
        Kont::SWait(v1, v, k) => State::ApplyV(*v1, *v, Some(Kont::BindV(Box::new(w), k))),
    }
}

pub fn new(t: Term) -> State {
    State::Eval(t, None)
}

impl State {
    pub fn step(self) -> Result<Self, Value> {
        match self {
            State::Eval(t, k) => Ok(eval(t, k)),
            State::ApplyT(v, t, k) => Ok(apply_t(v, t, k)),
            State::ApplyV(v, w, k) => Ok(apply_v(v, w, k)),
            State::ApplyK(Some(k), w) => Ok(apply_k(k, w)),
            State::ApplyK(None, v) => Err(v),
        }
    }

    pub fn run(self) -> Value {
        let mut state = self;
        loop {
            match state.step() {
                Ok(s) => state = s,
                Err(v) => return v,
            }
        }
    }
    // todo: add a repl
}

pub fn main(term: Term, interactive: bool) {
    let state = new(term);

    if interactive {
        let mut state = state;
        println!("{}", state);
        println!("Press enter to step, or Ctrl-C to exit. `r` to run to completion.");
        let result = loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "r" => break state.run(),
                _ => (),
            }
            match state.step() {
                Ok(s) => state = s,
                Err(v) => break v,
            }
            println!("{}", state);
        };
        println!("-----\nResult:\n{}", result);
    } else {
        let result = state.run();
        println!("Result:\n{}", result);
    }
}