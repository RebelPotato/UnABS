use crate::term::Term;
use std::char;
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;
use std::mem::take;

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

#[derive(Debug, Clone, PartialEq)]
pub enum StateFlag {
    Eval,
    ApplyT,
    ApplyV,
    ApplyK,
}

#[derive(Debug, Clone)]
pub struct State<'a> {
    flag: StateFlag,
    t: Option<&'a Term>, // this may bite me in the ass later
    v: Option<Rc<Value<'a>>>,
    w: Option<Rc<Value<'a>>>,
    k: Option<Rc<Kont<'a>>>,
}

impl Display for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.flag {
            StateFlag::Eval => {
                write!(f, "State: Eval\n")?;
            }
            StateFlag::ApplyT => {
                write!(f, "State: ApplyT\n")?;
            }
            StateFlag::ApplyV => {
                write!(f, "State: ApplyV\n")?;
            }
            StateFlag::ApplyK => {
                write!(f, "State: ApplyK\n")?;
            }
        }
        if let Some(v) = &self.v {
            write!(f, "Value: {}\n", v)?;
        }
        if let Some(t) = &self.t {
            write!(f, "Term: [{}]\n", t)?;
        }
        if let Some(w) = &self.w {
            write!(f, "Walue: {}\n", w)?;
        }
        match &self.k {
            None => write!(f, "Kont: ()"),
            Some(k) => write!(f, "Kont: {}", k),
        }
    }
}

fn eval<'a>(state: &mut State<'a>) {
    let t = state.t.unwrap();
    match t {
        Term::I => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::I0));
        }
        Term::S => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::S0));
        }
        Term::K => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::K0));
        }
        Term::V => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::V0));
        }
        Term::D => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::D0));
        }
        Term::C => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::C0));
        }
        Term::R => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::Put0('\n')));
        }
        Term::Put(c) => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::Put0(*c)));
        }
        Term::App(t0, t1) => {
            // State::Eval(t0, Some(Rc::new(Kont::BindT(t1, k.take()))))
            state.t = Some(t0);
            state.k = Some(Rc::new(Kont::BindT(t1, take(&mut state.k))));
        }
    };
}

fn apply_t<'a>(state: &mut State<'a>) {
    let v = take(&mut state.v).unwrap();
    // safe because clause1 has a value for v and clause2 gives a value to v
    match v.as_ref() {
        Value::D0 => {
            let t = state.t.unwrap();
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::D1T(t)));
        }
        _ => {
            state.flag = StateFlag::Eval;
            state.k = Some(Rc::new(Kont::BindV(v, take(&mut state.k))));
        }
    }
}

fn apply_v<'a>(state: &mut State<'a>) {
    let v = take(&mut state.v).unwrap();
    let w = take(&mut state.w).unwrap();
    match v.as_ref() {
        Value::I0 => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(w);
        }
        Value::Put0(c) => {
            print!("{}", c);
            std::io::stdout().flush().unwrap();
            state.flag = StateFlag::ApplyK;
            state.v = Some(w);
        }
        Value::K0 => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::K1(w)));
        }
        Value::K1(w0) => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(w0.clone());
        }
        Value::V0 => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(v);
        }
        Value::C0 => {
            state.v = Some(w);
            state.w = Some(Rc::new(Value::C1(state.k.clone())));
        }
        Value::C1(k1) => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(w);
            state.k = k1.clone();
        }
        Value::D0 => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::D1V(w)));
        }
        Value::D1T(t0) => {
            state.flag = StateFlag::Eval;
            state.t = Some(t0);
            state.k = Some(Rc::new(Kont::BindW(w, take(&mut state.k))));
        }
        Value::D1V(v0) => {
            state.v = Some(v0.clone());
            state.w = Some(w);
        }
        Value::S0 => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::S1(w)));
        }
        Value::S1(v0) => {
            state.flag = StateFlag::ApplyK;
            state.v = Some(Rc::new(Value::S2(v0.clone(), w)));
        }
        Value::S2(v0, v1) => {
            state.v = Some(v0.clone());
            state.w = Some(w.clone());
            state.k = Some(Rc::new(Kont::SWait(v1.clone(), w, take(&mut state.k))));
        }
    };
}

fn apply_k<'a>(state: &mut State<'a>) {
    let k = state.k.take();
    match k {
        Some(k) => match k.as_ref() {
            Kont::BindT(t, k) => {
                state.flag = StateFlag::ApplyT;
                state.t = Some(t);
                state.k = k.clone();
            }
            Kont::BindV(v, k) => {
                let w = take(&mut state.v).unwrap();
                state.flag = StateFlag::ApplyV;
                state.v = Some(v.clone());
                state.w = Some(w);
                state.k = k.clone();
            }
            Kont::BindW(w1, k) => {
                state.flag = StateFlag::ApplyV;
                state.w = Some(w1.clone());
                state.k = k.clone();
            }
            Kont::SWait(v1, v, k) => {
                let w = take(&mut state.v).unwrap();
                state.flag = StateFlag::ApplyV;
                state.v = Some(v1.clone());
                state.w = Some(v.clone());
                state.k = Some(Rc::new(Kont::BindV(w, k.clone())));
            }
        },
        None => ()
    }
}

pub fn new<'a>(t: &'a Term) -> State<'a> {
    State {
        flag: StateFlag::Eval,
        t: Some(t),
        v: None,
        w: None,
        k: None,
    }
}

impl<'a> State<'a> {
    pub fn step(&mut self) {
        match self.flag {
            StateFlag::Eval => eval(self),
            StateFlag::ApplyT => apply_t(self),
            StateFlag::ApplyV => apply_v(self),
            StateFlag::ApplyK => apply_k(self)
        }
    }

    pub fn extract(&self) -> Option<Rc<Value<'a>>> {
        if self.flag == StateFlag::ApplyK && self.k.is_none() {
            self.v.clone()
        } else {
            None
        }
    }

    pub fn run(self) -> Rc<Value<'a>> {
        let mut state = self;
        loop {
            state.step();
            if let Some(v) = state.extract() {
                return v;
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
            state.step();
            if let Some(v) = state.extract() {
                break v;
            }
            println!("{}", state);
        };
        println!("-----\nResult:\n{}", result);
    } else {
        let result = state.run();
        println!("Result:\n{}", result);
    }
}
