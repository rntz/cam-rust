#![allow(dead_code)]
use std::ops::Add;

pub struct Rope<A>(Tree<A>);

// Internals
enum Tree<A> {
    Empty,
    Leaf(Vec<A>),               // invariant: never empty
    Node(Vec<Tree<A>>),         // invariant: never empty
}
use self::Tree::*;

impl<A> Default for Rope<A> { fn default() -> Rope<A> { Rope(Empty) } }

impl<A> Add for Rope<A> {
    type Output = Rope<A>;
    fn add(self, rhs: Rope<A>) -> Rope<A> {
        Rope(match self.0 {
                Empty => rhs.0,
                Leaf(..) => Node(vec![self.0, rhs.0]),
                Node(mut v) => { v.push(rhs.0); Node(v) }
            })
    }
}

impl<A> Rope<A> {
    fn one(x: A) -> Rope<A> { Rope(Leaf(vec![x])) }
    fn from_vec(v: Vec<A>) -> Rope<A> { Rope(Leaf(v)) }
    fn concat(&self) -> Vec<A> {
        unimplemented!()
    }
}
