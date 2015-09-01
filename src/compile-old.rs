#![allow(dead_code,unused_imports)]

use lang::*;
use vm::*;

use vm::Instr::*;
use vm::Do::*;

struct State {
    instrs: Vec<Instr>,
    // maps DeBruijn indices onto registers, in reverse order (eg. DeBruijn
    // index 0 refers to the last element of `env')
    env: Vec<Reg>,
}

impl State {
    fn get_var(&self, idx: u32) -> Reg {
        let i = idx as usize;
        assert!(i < self.env.len());
        self.env[self.env.len() - i - 1]
    }

    fn compute(&mut self, e: &Exp) -> Do {
        use vm::Src::*;
        match *e {
            Exp::Lit(ref l) => Load(Lit(l.clone())),
            Exp::Var(_, idx) => Load(Reg(self.get_var(idx))),
            Exp::Lam(ref _params, ref _body) => unimplemented!(),
            Exp::App(ref _func, ref _args) => unimplemented!(),
            Exp::Let(ref _binds, ref _body) => unimplemented!(),
        }
    }

    fn compile(&mut self, dest: Dest, exp: &Exp) {
        let e = self.compute(exp);
        self.instrs.push(Put(dest, e));
    }
}
