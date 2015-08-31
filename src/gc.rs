#![allow(dead_code)]
// #![feature(alloc)]
// extern crate alloc;

use std::ops::Deref;

// Gc-able pointer.
pub struct Gc<T> { _ptr: *mut T }

impl<T: Sized> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self._ptr } }
}

// Trait for GC-able things.
pub trait GC { fn scan(&mut self, ctx: &Context); }

// GC context - keeps relevant info during a GC cycle.
// TODO: implement.
pub struct Context;
impl Context {
    pub fn visit<T>(&self, _p: &mut Gc<T>) { unimplemented!() }
}

// Allocating shit.
pub struct Heap;

impl Heap {
    pub fn new<T: Sized>(&mut self) -> Gc<T> {
        unimplemented!()
    }
}

// // Example:
// enum Tree { Empty, Node(String, Gc<Tree>, Gc<Tree>) }

// impl GC for Tree {
//     fn scan(&mut self, ctx: &Context) {
//         match *self {
//             Tree::Empty => (),
//             Tree::Node(_, ref mut left, ref mut right) => {
//                 ctx.visit(left);
//                 ctx.visit(right); } } } }
