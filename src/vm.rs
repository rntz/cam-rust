use std::mem;
use std::rc::{Rc};

use lang::*;

// TODO: implement, somehow.
fn rc_eq<T>(_a: &Rc<T>, _b: &Rc<T>) -> bool {
    panic!("reference equality not implemented")
}

// ---------- Virtual machine ----------
pub type Reg = u32;
pub type Upvar = u32;
pub type SubfunctionIndex = u32;
pub type Index = u32;    // index into instruction array (instruction "address")

#[derive(Clone)]
pub enum Val {
    Lit(Lit),
    // Cell(RefCell<Option<Val>>),
    // TODO?: should be a gc'ed pointer?
    Func(Rc<Proto>, Rc<Upvars>),
}
pub type Upvars = Vec<Val>;

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
    pub fn as_int(&self) -> i64 {
        match *self { Val::Lit(Lit::Int(i)) => i,
                      _ => panic!("value not an int") }
    }
    pub fn as_string(&self) -> Rc<String> {
        match *self { Val::Lit(Lit::String(ref s)) => s.clone(),
                      _ => panic!("value not a string") }
    }

    pub fn truthy(&self) -> bool {
        match *self { Val::Lit(Lit::Nil) => false,
                      Val::Lit(Lit::Bool(false)) => false,
                      _ => true }
    }
}

// A function prototype
pub struct Proto {
    pub instrs: Vec<Instr>,
    pub arity: Arity,
    pub subfunctions: Vec<Rc<Proto>>,
}

#[derive(Clone)]
pub enum Src { Temp, Lit(Lit), Reg(Reg), Upvar(Upvar), Cell(Upvar) }
#[derive(Clone)]
pub enum Do {
    Load(Src),
    Call(Src, Reg, Arity),
    Close(SubfunctionIndex, Vec<Src>),
}

#[derive(Clone,Copy)]
pub enum Dest { Temp, Ignore, Reg(Reg), If(Index), Ifnot(Index), Return }
pub enum Instr {
    Put(Dest, Do),
    Jump(u32),
}

pub struct Frame {
    pub proto: Rc<Proto>,
    pub env: Rc<Upvars>,
    pub index: Index,
    pub reg_base: usize,
}

pub struct VM {
    pub frame: Frame,
    pub frames: Vec<Frame>,
    pub regs: Vec<Val>,
    pub temp: Option<Val>,
}

impl VM {
    pub fn step(&mut self) {
        let index = self.frame.index as usize;
        assert!(index < self.frame.proto.instrs.len());
        let (dest, exp) = match self.frame.proto.instrs[index] {
            Instr::Jump(j) => { self.frame.index = j; return; }
            Instr::Put(dest, ref exp) => (dest, exp.clone())
        };

        let val = match exp {
            Do::Load(ref src) => self.load(src),
            Do::Close(subfn, ref srcs) => self.closure(subfn, srcs),
            Do::Call(ref src, reg, arity) =>
                match self.call(src, reg, arity) {
                    Some(result) => result,
                    None => return
                }
        };
        self.temp = None;       // no longer needed

        self.step_finish(dest, val);
    }

    pub fn step_finish(&mut self, dest: Dest, val: Val) {
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

    pub fn set_reg(&mut self, reg: Reg, val: Val) {
        // FIXME: needs to check whether self.regs is long enough!
        self.regs[self.frame.reg_base + reg as usize] = val;
    }

    pub fn ret(&mut self, val: Val) {
        self.frame = self.frames.pop().unwrap();

        // Analyze the destination and put the return value into it.
        assert!((self.frame.index as usize) < self.frame.proto.instrs.len());
        match self.frame.proto.instrs[self.frame.index as usize] {
            Instr::Put(dest, _) => self.step_finish(dest, val),
            _ => panic!("return to non-Put instruction"),
        }
    }

    pub fn load(&self, src: &Src) -> Val {
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

    pub fn closure(&self, subfn: SubfunctionIndex, upvars: &Vec<Src>) -> Val {
        Val::Func(self.frame.proto.subfunctions[subfn as usize].clone(),
                  Rc::new(upvars.iter().map(|x| self.load(x)).collect()))
    }

    // returns Option<Val> because we might be calling into a function, in which
    // case we change our state and return None, or we might be calling a
    // primitive, in which case we return Just(result).
    pub fn call(&mut self, func_src: &Src, first_arg: Reg, num_args: Arity)
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

    pub fn call_func(&mut self, proto: Rc<Proto>, env: Rc<Upvars>,
                     first_arg: Reg, num_args: Arity) {
        if num_args != proto.arity { panic!("wrong # args to function"); }

        let new_reg_base = self.frame.reg_base + first_arg as usize;
        self.frames.push(
            mem::replace(&mut self.frame,
                         Frame { proto: proto, env: env, index: 0,
                                 reg_base: new_reg_base }));
    }

    pub fn call_prim(&self, prim: Prim, first_arg: Reg, num_args: Arity) -> Val {
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

    pub fn load_cell(&self, _v: &Val) -> Val {
        unimplemented!()
    }
}
