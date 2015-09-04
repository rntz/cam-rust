// TODO: a way to create and initialize a VM.
use std::rc::Rc;
use std::mem;

pub type InstrIndex = u16;

use lang::*;
use string::Str;

#[derive(Clone,Debug)]
pub enum Val { Lit(Lit), Func(Func) }
type Env = Vec<Val>;

impl Val {
    pub fn truthy(&self) -> bool {
        match self { &Val::Lit(ref l) => l.truthy(),
                     &Val::Func(..) => true }
    }
    pub fn as_int(&self) -> i64 {
        match *self { Val::Lit(Lit::Int(x)) => x,
                      _ => panic!("non-integer value") }
    }
    pub fn as_string(&self) -> Str {
        match *self { Val::Lit(Lit::String(ref x)) => x.clone(),
                      _ => panic!("non-string value") }
    }
}
impl PartialEq for Val {
    fn eq(&self, other: &Val) -> bool {
        match (self, other) {
            (&Val::Lit(ref a), &Val::Lit(ref b)) => a == b,
            // don't ask for equality on functions. just don't.
            (&Val::Func(_), &Val::Func(_)) => unimplemented!(),
            _ => false
        }
    }
}
impl Eq for Val {}

#[derive(Clone,Debug)]
pub struct Func { proto: Rc<Proto>, env: Rc<Env> }
#[derive(Debug)]
pub struct Proto { pub code: Code, pub arity: Arity }

pub type Code = Vec<Instr>;
#[derive(Debug)]
pub enum Instr {
    Access(VarIndex),
    Push(Lit),
    Apply(Arity), TailApply(Arity),
    Closure(Rc<Proto>),
    If(InstrIndex, InstrIndex),
    Jump(InstrIndex),
    Return,
    // Let, // optimization
    // Prim(Prim), // optimization
}
// TODO: examine mem::{size,align}_of::<Instr>()

#[derive(Debug)]
pub struct VM {
    stack: Vec<Val>,
    frame: Frame,
    frames: Vec<Frame>
}
#[derive(Debug)]
struct Frame {
    proto: Rc<Proto>,
    ip: usize,
    env: FrameEnv,
}
#[derive(Debug)]
struct FrameEnv {
    // This could perhaps be more efficiently accomplished by using is_unique on
    // an Rc<Env>, but that's still unstable.
    // (edit) Actually, could we use borrow::Cow for this?
    shared: Rc<Env>,
    unique: Env,                // variables not yet closed over
}

impl FrameEnv {
    fn access(&self, index: VarIndex) -> Val {
        let i = 1 + index as usize;
        let u_len = self.unique.len();
        if i <= u_len { return self.unique[u_len - i].clone() }
        let len = u_len + self.shared.len();
        assert!(i <= len);
        self.shared[len - i].clone()
    }
    fn close(&mut self) -> Rc<Env> {
        if self.unique.is_empty() { return self.shared.clone() }
        let mut env: Env = (*self.shared).clone();
        // This wants to be a vec::append, but that's unstable.
        env.extend(mem::replace(&mut self.unique, vec![]).into_iter());
        self.shared = Rc::new(env);
        self.shared.clone()
    }
}

impl VM {
    pub fn run(code: Code) -> Val {
        let mut vm = VM {
            stack: vec![],
            frames: vec![],
            frame: Frame {
                proto: Rc::new(Proto{code: code, arity: 0}),
                ip: 0,
                env: FrameEnv{shared: Rc::new(vec![]), unique: vec![]}
            }
        };
        while !vm.done() { vm.step() }
        vm.value()
    }

    // only to be used within `run'
    fn done(&self) -> bool {
        // TODO: could do this more effectively with frames trickery.
        self.frames.is_empty() &&
            (self.frame.ip == self.frame.proto.code.len()
             || match self.frame.proto.code[self.frame.ip] {
                Instr::Return | Instr::TailApply(..) => true,
                _ => false
            })
    }

    fn value(mut self) -> Val {
        debug_assert!(self.stack.len() == 1);
        self.stack.pop().unwrap()
    }

    pub fn step(&mut self) {
        use self::Instr::*;

        // avoids borrowing complications at the expense of a refcount bump.
        let proto = self.frame.proto.clone();
        let ip = self.frame.ip;
        self.frame.ip += 1;
        assert!(ip < proto.code.len());

        if ::DEBUG {
            println!(" instr:  {:?}
 stack:  {:?}
 env:    {:?}
 frames: {:?}
",
                     proto.code[ip],
                     self.stack,
                     self.frame.env,
                     self.frames)
        }

        match proto.code[ip] {
            Access(i) => { let val = self.frame.env.access(i);
                           self.stack.push(val) }
            Push(ref l) => self.stack.push(Val::Lit(l.clone())),
            Closure(ref proto) =>
                self.stack.push(Val::Func(Func {
                        proto: proto.clone(),
                        env: self.frame.env.close() })),
            If(thn, els) =>
                (self.frame.ip =
                    if self.stack.pop().unwrap().truthy() { thn }
                    else { els } as usize),
            Jump(index) => (self.frame.ip = index as usize),
            Apply(arity) => self.apply(arity, false),
            TailApply(arity) => self.apply(arity, true),
            Return => self.ret(),
        }
    }

    fn apply(&mut self, arity: Arity, tail: bool) {
        let num_vals = 1 + arity as usize;
        assert!(num_vals <= self.stack.len());
        let func_idx = self.stack.len() - num_vals;
        let func = self.stack[func_idx].clone();
        match func {
            Val::Func(f) => self.call(f, arity, func_idx, tail),
            Val::Lit(Lit::Prim(prim)) => {
                let val = self.prim(prim, &self.stack[func_idx+1..]);
                self.stack.truncate(func_idx);
                self.stack.push(val);
            }
            _ => panic!("applying non-function"),
        }
    }

    #[inline]
    fn call(&mut self, func: Func, arity: Arity, func_idx: usize, tail: bool) {
        debug_assert!(arity as usize == self.stack.len() - func_idx - 1);
        if arity != func.proto.arity {
            panic!("wrong number of arguments to function")
        }
        let new_frame = Frame {
            proto: func.proto.clone(),
            ip: 0,
            env: FrameEnv {
                shared: func.env.clone(),
                unique: self.stack[func_idx+1..].to_vec(),
            }
        };
        self.stack.truncate(func_idx);
        if !tail {
            self.frames.push(mem::replace(&mut self.frame, new_frame));
        } else {
            // Drop our old frame.
            self.frame = new_frame;
        }
    }

    #[inline]
    fn prim(&self, prim: Prim, args: &[Val]) -> Val {
        if args.len() != prim.arity() as usize {
            panic!("applying prim to wrong # of arguments")
        }
        Val::Lit(match prim {
                Equal => Lit::Bool(args[0] == args[1]),
                Add => Lit::Int(args[0].as_int() + args[1].as_int()),
                Sub => Lit::Int(args[0].as_int() - args[1].as_int()),
                Mul => Lit::Int(args[0].as_int() * args[1].as_int()),
                Div => Lit::Int(args[0].as_int() / args[1].as_int()),
                Print => {
                    println!("{}", args[0].as_string());
                    Lit::Nil
                }
            })
    }

    fn ret(&mut self) {
        debug_assert!(!self.frames.is_empty());
        self.frame = self.frames.pop().unwrap();
        // do we need to advance ip? no.
    }
}
