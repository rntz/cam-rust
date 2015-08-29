#![allow(dead_code)]            // for now

use std::mem;
use std::rc::{Rc};
use std::cmp::{Eq};

// TODO
fn rc_eq<T>(_a: &Rc<T>, _b: &Rc<T>) -> bool {
    panic!("reference equality not implemented")
}

// TODO: lookaside table recording which registers are in use at which time, for
// the GC.

// TODO: exceptions & exception handling

// TODO: interned strings? Gc'ed strings?

type Arity = u32;
type Ident = Rc<String>;

#[derive(Clone,Copy,PartialEq,Eq)]
enum Prim { Equal, Add, Sub, Mul, Div, Print }
use Prim::*;
impl Prim {
    fn arity(&self) -> Arity {
        match *self {
            Equal | Add | Sub | Mul | Div => 2,
            Print => 1,
        }
    }
}

#[derive(Clone,PartialEq,Eq)]
enum Lit { Nil, Bool(bool), Int(i64), String(Rc<String>), Prim(Prim) }

type Expr = Box<Exp>;
enum Exp {
    Lit(Lit),
    Var(Ident, u32),
    Lam(Vec<Ident>, Expr),
    App(Expr, Vec<Exp>),
    // simultaneous binding; no let-bound expression sees any of the others.
    Let(Vec<(Ident,Exp)>, Expr),
}

// Compiling expressions to VM code.
// fn compile(e: &Exp, offset: Index) -> Vec<Instr> {
// }


// ---------- Virtual machine ----------
type Reg = u32;
type Upvar = u32;
type SubfunctionIndex = u32;
type Index = u32;        // index into instruction array (instruction "address")

#[derive(Clone)]
enum Val {
    Lit(Lit),
    // Cell(RefCell<Option<Val>>),
    // TODO?: should be a gc'ed pointer?
    Func(Rc<Proto>, Rc<Env>),
}
type Env = Vec<Val>;

impl PartialEq for Val {
    fn eq(&self, other: &Val) -> bool {
        match (self, other) {
            (&Val::Lit(ref x), &Val::Lit(ref y)) => x == y,
            (&Val::Func(ref p1,ref e1), &Val::Func(ref p2,ref e2)) =>
                rc_eq(p1,p2) && rc_eq(e1,e2),
            _ => false
        }
    }
}
impl Eq for Val {}
impl Val {
    fn as_int(&self) -> i64 {
        match *self { Val::Lit(Lit::Int(i)) => i,
                      _ => panic!("value not an int") }
    }
    fn as_string(&self) -> Rc<String> {
        match *self { Val::Lit(Lit::String(ref s)) => s.clone(),
                      _ => panic!("value not a string") }
    }

    fn truthy(&self) -> bool {
        match *self { Val::Lit(Lit::Nil) => false,
                      Val::Lit(Lit::Bool(false)) => false,
                      _ => true }
    }
}

// A function prototype
struct Proto {
    instrs: Vec<Instr>,
    arity: Arity,
    subfunctions: Vec<Rc<Proto>>,
}

#[derive(Clone)]
enum Src { Temp, Lit(Lit), Reg(Reg), Upvar(Upvar), Cell(Upvar) }
#[derive(Clone)]
enum InstrExp {
    Load(Src),
    Call(Src, Reg, Arity),
    Closure(SubfunctionIndex, Vec<Src>),
}

#[derive(Clone,Copy)]
enum Dest { Temp, Ignore, Reg(Reg), If(Index), Ifnot(Index), Return }
enum Instr {
    Put(Dest, InstrExp),
    Jump(u32),
}

#[derive(Clone)]                // ugh
struct Frame {
    proto: Rc<Proto>,
    env: Rc<Env>,
    index: Index,
    reg_base: usize,
}

struct VM {
    frame: Frame,
    frames: Vec<Frame>,
    regs: Vec<Val>,
    temp: Option<Val>,
}

impl VM {
    fn step(&mut self) {
        let index = self.frame.index as usize;
        assert!(index < self.frame.proto.instrs.len());
        let (dest, exp) = match self.frame.proto.instrs[index] {
            Instr::Jump(j) => { self.frame.index = j; return; }
            Instr::Put(dest, ref exp) => (dest, exp.clone())
        };

        let val = match exp {
            InstrExp::Load(ref src) => self.load(src),
            InstrExp::Closure(subfn, ref srcs) => self.closure(subfn, srcs),
            InstrExp::Call(ref src, reg, arity) =>
                match self.call(src, reg, arity) {
                    Some(result) => result,
                    None => return
                }
        };
        self.temp = None;       // no longer needed

        self.step_finish(dest, val);
    }

    fn step_finish(&mut self, dest: Dest, val: Val) {
        match dest {
            Dest::Temp => { self.temp = Some(val); }
            Dest::Ignore => {}
            Dest::Reg(r) => self.set_reg(r, val),
            Dest::If(i) => if val.truthy() { self.frame.index = i; return; },
            Dest::Ifnot(i) => if !val.truthy() {
                self.frame.index = i; return;
            },
            Dest::Return => { self.ret(val); return; }
        }

        self.frame.index += 1;
    }

    fn set_reg(&mut self, reg: Reg, val: Val) {
        // FIXME: needs to check whether self.regs is long enough!
        self.regs[self.frame.reg_base + reg as usize] = val;
    }

    fn ret(&mut self, val: Val) {
        self.frame = self.frames.pop().unwrap();

        // Analyze the destination and put the return value into it.
        assert!((self.frame.index as usize) < self.frame.proto.instrs.len());
        match self.frame.proto.instrs[self.frame.index as usize] {
            Instr::Put(dest, _) => self.step_finish(dest, val),
            _ => panic!("return to non-Put instruction"),
        }
    }

    fn load(&self, src: &Src) -> Val {
        match *src {
            Src::Temp => match self.temp {
                Some(ref x) => x.clone(),
                None => panic!("temp not set")
            },
            Src::Lit(ref lit) => Val::Lit(lit.clone()),
            Src::Reg(r) => self.regs[r as usize].clone(),
            Src::Upvar(r) => self.frame.env[r as usize].clone(),
            Src::Cell(r) => self.load_cell(&self.frame.env[r as usize]),
        }
    }

    fn closure(&self, subfn: SubfunctionIndex, upvars: &Vec<Src>) -> Val {
        Val::Func(self.frame.proto.subfunctions[subfn as usize].clone(),
                  Rc::new(upvars.iter().map(|x| self.load(x)).collect()))
    }

    // returns Option<Val> because we might be calling into a function, in which
    // case we change our state and return None, or we might be calling a
    // primitive, in which case we return Just(result).
    fn call(&mut self, func_src: &Src, first_arg: Reg, num_args: Arity)
            -> Option<Val> {
        match self.load(func_src) {
            Val::Func(proto, env) => {
                self.call_func(proto, env, first_arg, num_args);
                None
            }
            Val::Lit(Lit::Prim(p)) =>
                Some(self.call_prim(p, first_arg, num_args)),
            _ => panic!("calling non-function"),
        }
    }

    fn call_func(&mut self, proto: Rc<Proto>, env: Rc<Env>, first_arg: Reg,
                 num_args: Arity) {
        if num_args != proto.arity { panic!("wrong # args to function"); }

        let new_reg_base = self.frame.reg_base + first_arg as usize;
        self.frames.push(
            mem::replace(&mut self.frame,
                         Frame { proto: proto, env: env, index: 0,
                                 reg_base: new_reg_base }));
    }

    fn call_prim(&self, prim: Prim, first_arg: Reg, num_args: Arity) -> Val {
        if num_args != prim.arity() {
            panic!("wrong # of arguments to primitive");
        }
        let args: Vec<Val> =
            (first_arg .. first_arg + num_args)
            .map(|i| { self.load(&Src::Reg(i)) })
            .collect();
        Val::Lit(match prim {
                Equal => Lit::Bool(args[0] == args[1]),
                Add => Lit::Int(args[0].as_int() + args[1].as_int()),
                Sub => Lit::Int(args[0].as_int() - args[1].as_int()),
                Mul => Lit::Int(args[0].as_int() * args[1].as_int()),
                Div => Lit::Int(args[0].as_int() / args[1].as_int()),
                Print => { println!("{}", args[0].as_string()); Lit::Nil }
            })
    }

    fn load_cell(&self, _v: &Val) -> Val {
        unimplemented!()
    }
}

// fn do_call(src: &Src, _reg: Reg, _arity: Arity, temp: &Val, temp_is_set: bool, env: &Env, regs: &Vec<Val>) -> Val {
//     let (_func_proto, _func_env) = match do_load(src, temp, temp_is_set, env, regs) {
//         Val::Func(ref proto, ref env) => (proto.clone(), env.clone()),
//         _ => panic!("calling a non-function"),
//     };
//     panic!("calling things unimplemented");
// }

// enum Src { Prev, Lit(Lit), Reg(Reg), Upvar(Upvar), Cell(Upvar) }
// enum Dest { Return, Reg(Reg) }
// enum Instr {
//     Put(Dest, InstrExp),
//     Jump(u32),
//     Branch(Src, u32, u32),
// }

fn main() {}
