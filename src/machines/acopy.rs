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
            Kont::SWait(w0, w1, k) => {
                write!(f, "SWait({}; {})", w0, w1)?;
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
        Value::D0 => State::ApplyKont(k, Value::D1T(Box::new(t))),
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
        Value::K0 => State::ApplyKont(k, Value::K1(Box::new(w))),
        Value::K1(w0) => State::ApplyKont(k, *w0),
        Value::V0 => State::ApplyKont(k, Value::V0),
        // This clones the kontinuation. How can we avoid this?
        Value::C0 => State::ApplyV(w, Value::C1(Box::new(k.clone())), k),
        Value::C1(k1) => State::ApplyKont(*k1, w),
        Value::D0 => State::ApplyKont(k, Value::D1V(Box::new(w))),
        Value::D1T(t0) => State::Eval(*t0, Some(Kont::BindW(Box::new(w), Box::new(k)))),
        Value::D1V(v0) => State::ApplyV(*v0, w, k),
        Value::S0 => State::ApplyKont(k, Value::S1(Box::new(w))),
        Value::S1(v0) => State::ApplyKont(k, Value::S2(v0, Box::new(w))),
        Value::S2(v0, v1) => {
            // This copys the third value. A tree clone! Very inefficient.
            // How do we share? Rc? Cow? Make a flat list or something?
            State::ApplyV(*v0, w.clone(), Some(Kont::SWait(v1, Box::new(w), Box::new(k))))
        }
    }
}

fn apply_kont(k: Kont, w: Value) -> State {
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
            State::ApplyKont(Some(k), w) => Ok(apply_kont(k, w)),
            State::ApplyKont(None, v) => Err(v),
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
