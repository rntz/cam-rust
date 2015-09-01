#![allow(dead_code,unused_imports,unused_variables)]

use std::rc::Rc;

use lang::*;
use cam::*;

fn compile(e: &Exp) -> Vec<Instr> {
    let mut s = State { instrs: vec![] };
    s.compile(e);
    s.instrs
}

struct State {
    instrs: Vec<Instr>,
}

impl State {
    fn compile(&mut self, e: &Exp) {
        use cam::Instr::*;
        match *e {
            Exp::Lit(ref l) => self.instrs.push(Push(l.clone())),
            Exp::Var(_, index) => self.instrs.push(Access(index)),
            Exp::Lam(ref ids, ref body) => {
                let proto = Proto { instrs: compile(body),
                                    arity: ids.len() as Arity };
                self.instrs.push(Closure(Rc::new(proto)));
            }
            Exp::App(ref func, ref args) => {
                self.compile(func);
                for arg in args { self.compile(arg) }
                self.instrs.push(Apply(args.len() as Arity))
            }
            Exp::Let(ref binds, ref body) => {
                // TODO: better compilation strategy here.
                let proto = Proto { instrs: compile(body),
                                    arity: binds.len() as Arity };
                self.instrs.push(Closure(Rc::new(proto)));
                for &(_, ref exp) in binds { self.compile(exp) }
                self.instrs.push(Apply(binds.len() as Arity))
            }
            Exp::If(ref subject, ref thn, ref els) => {
                self.compile(subject);

                // We push an If with dummy addresses and fix it up later.
                let if_index = self.instrs.len();
                self.instrs.push(If(0, 0));
                let thn_index = self.instrs.len() as InstrIndex;
                self.compile(thn);
                let jmp_index = self.instrs.len();
                self.instrs.push(Jump(0)); // again, we fix it up later
                let els_index = self.instrs.len() as InstrIndex;
                self.compile(els);
                let join_index = self.instrs.len() as InstrIndex;

                // Fix up our jumps.
                self.instrs[if_index] = If(thn_index, els_index);
                self.instrs[jmp_index] = Jump(join_index);
            }
        }
    }
}
