use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Borrow;
use std::fmt::{Display,Formatter,Error};

// Ugh. I hate having to do this, but.
#[derive(Debug,Hash,PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct Str(Rc<String>);

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool { &**self == other }
}

impl Str {
    pub fn new(s: &str) -> Str { Str(Rc::new(String::from(s))) }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &str { self.0.deref() }
}

impl Borrow<str> for Str { fn borrow(&self) -> &str { &*self } }

impl Display for Str {
    fn fmt(&self, f: &mut Formatter) -> Result<(),Error> {
        let s: &str = self;
        s.fmt(f)
    }
}
