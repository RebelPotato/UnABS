use crate::term::Term;
use std::char;
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

// A sharing abstract machine for unlambda
// I hope this runs f..a..s..t..

#[derive(Debug, Clone)]
pub enum Value<'a> {
    I0,
    S0,
    K0,
    V0,
    D0,
    C0,
    Put0(char),
    S1(Rc<Value<'a>>),
    S2(Rc<Value<'a>>, Rc<Value<'a>>),
    K1(Rc<Value<'a>>),
    D1T(&'a Term),
    D1V(Rc<Value<'a>>),
    C1(Option<Rc<Kont<'a>>>),
}

impl Display for Value<'_> {
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
pub enum Kont<'a> {
    BindT(&'a Term, Option<Rc<Kont<'a>>>),
    BindV(Rc<Value<'a>>, Option<Rc<Kont<'a>>>),
    BindW(Rc<Value<'a>>, Option<Rc<Kont<'a>>>),
    SWait(Rc<Value<'a>>, Rc<Value<'a>>, Option<Rc<Kont<'a>>>),
}
impl Display for Kont<'_> {
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

pub enum State<'a> {
    Eval(&'a Term, Option<Rc<Kont<'a>>>),
    ApplyT(Rc<Value<'a>>, &'a Term, Option<Rc<Kont<'a>>>),
    ApplyV(Rc<Value<'a>>, Rc<Value<'a>>, Option<Rc<Kont<'a>>>),
    ApplyKont(Option<Rc<Kont<'a>>>, Rc<Value<'a>>),
}

impl Display for State<'_> {
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
                match k.as_ref() {
                    None => write!(f, "Kont: ()"),
                    Some(k) => write!(f, "Kont: {}", k),
                }
            }
        }
    }
}

fn eval<'a>(t: &'a Term, k: Option<Rc<Kont<'a>>>) -> State<'a> {
    match t {
        Term::I => State::ApplyKont(k, Value::I0.into()),
        Term::S => State::ApplyKont(k, Value::S0.into()),
        Term::K => State::ApplyKont(k, Value::K0.into()),
        Term::V => State::ApplyKont(k, Value::V0.into()),
        Term::D => State::ApplyKont(k, Value::D0.into()),
        Term::C => State::ApplyKont(k, Value::C0.into()),
        Term::R => State::ApplyKont(k, Value::Put0('\n').into()),
        Term::Put(c) => State::ApplyKont(k, Value::Put0(*c).into()),
        Term::App(t0, t1) => State::Eval(t0.as_ref(), Some(Kont::BindT(t1.as_ref(), k).into())),
    }
}

fn apply_t<'a>(v: Rc<Value<'a>>, t: &'a Term, k: Option<Rc<Kont<'a>>>) -> State<'a> {
    match v.as_ref() {
        Value::D0 => State::ApplyKont(k, Value::D1T(t).into()),
        _ => State::Eval(t, Some(Kont::BindV(v, k).into())),
    }
}

fn apply_v<'a>(v: Rc<Value<'a>>, w: Rc<Value<'a>>, k: Option<Rc<Kont<'a>>>) -> State<'a> {
    match v.as_ref() {
        Value::I0 => State::ApplyKont(k, w),
        Value::Put0(c) => {
            print!("{}", c);
            std::io::stdout().flush().unwrap();
            State::ApplyKont(k, w)
        }
        Value::K0 => State::ApplyKont(k, Value::K1(w).into()),
        Value::K1(w0) => State::ApplyKont(k, w0.clone()),
        Value::V0 => State::ApplyKont(k, Value::V0.into()),
        Value::C0 => State::ApplyV(w, Value::C1(k.clone()).into(), k),
        Value::C1(k1) => State::ApplyKont(k1.clone(), w),
        Value::D0 => State::ApplyKont(k, Value::D1V(w).into()),
        Value::D1T(t0) => State::Eval(t0, Some(Kont::BindW(w, k).into())),
        Value::D1V(v0) => State::ApplyV(v0.clone(), w, k),
        Value::S0 => State::ApplyKont(k, Value::S1(w).into()),
        Value::S1(v0) => State::ApplyKont(k, Value::S2(v0.clone(), w).into()),
        Value::S2(v0, v1) => State::ApplyV(
            v0.clone(),
            Rc::clone(&w),
            Some(Kont::SWait(v1.clone(), w, k).into()),
        ),
    }
}

fn apply_kont<'a>(k: Rc<Kont<'a>>, w: Rc<Value<'a>>) -> State<'a> {
    match k.as_ref() {
        Kont::BindT(t, k) => State::ApplyT(w, t, k.clone()),
        Kont::BindV(v, k) => State::ApplyV(v.clone(), w, k.clone()),
        Kont::BindW(w1, k) => State::ApplyV(w, w1.clone(), k.clone()),
        Kont::SWait(v1, v, k) => State::ApplyV(
            v1.clone(),
            v.clone(),
            Some(Kont::BindV(w, k.clone()).into()),
        ),
    }
}

pub fn new<'a>(t: &'a Term) -> State<'a> {
    State::Eval(t, None)
}

impl<'a> State<'a> {
    pub fn step(self) -> Result<Self, Rc<Value<'a>>> {
        match self {
            State::Eval(t, k) => Ok(eval(t, k)),
            State::ApplyT(v, t, k) => Ok(apply_t(v, t, k)),
            State::ApplyV(v, w, k) => Ok(apply_v(v, w, k)),
            State::ApplyKont(Some(k), w) => Ok(apply_kont(k, w)),
            State::ApplyKont(None, v) => Err(v),
        }
    }

    pub fn run(self) -> Rc<Value<'a>> {
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
    let state = new(&term);

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
