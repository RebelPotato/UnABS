// use std::fmt::Display;
// use crate::term::{CompiledTerm, Instr, Primitive};

// // virtual machine with a compile step

// #[derive(Debug, Clone)]
// struct Term(Vec<Instr>);

// #[derive(Debug, Clone)]
// pub enum Value {
//     I0,
//     S0,
//     K0,
//     V0,
//     D0,
//     C0,
//     Put0(char),
//     S1(Box<Value>),
//     S2(Box<Value>, Box<Value>),
//     K1(Box<Value>),
//     D1T(Box<Term>),
//     D1V(Box<Value>),
//     C1(Box<Option<Kont>>),
// }

// impl Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::I0 => write!(f, "i"),
//             Value::S0 => write!(f, "s"),
//             Value::K0 => write!(f, "k"),
//             Value::V0 => write!(f, "v"),
//             Value::D0 => write!(f, "d"),
//             Value::C0 => write!(f, "c"),
//             Value::Put0(c) => {
//                 if *c == '\n' {
//                     write!(f, "r")
//                 } else {
//                     write!(f, ".{}", c)
//                 }
//             }
//             Value::S1(w) => write!(f, "`s{}", w),
//             Value::S2(w0, w1) => write!(f, "``s{}{}", w0, w1),
//             Value::K1(w) => write!(f, "`k{}", w),
//             Value::D1T(t) => write!(f, "`d[{}]", t),
//             Value::D1V(t) => write!(f, "`d{}", t),
//             Value::C1(k) => match k.as_ref() {
//                 Some(k) => write!(f, "`c({})", k),
//                 None => write!(f, "`c()"),
//             },
//         }
//     }
// }

// pub struct KontStack(Vec<Kont>);
// pub enum Kont {
//     BindT(Box<Term>),
//     BindV(Box<Value>),
//     BindW(Box<Value>),
//     SWait(Box<Value>),
// }