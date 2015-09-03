#![allow(unused_variables,dead_code,unused_imports)]

use std::collections::{HashSet,HashMap};
use std::str::Chars;
use std::rc::Rc;
use std::borrow::Borrow;

use regex::Regex;

#[derive(Clone,Debug)]
pub enum Sexp {
    Int(i64),
    String(Rc<String>),
    Symbol(Rc<String>),
    List(Vec<Sexp>),
}
// TODO: impl Display for Sexp<'a>

// we can't get keys out of hashsets by looking them up, so we have to use an
// identity hashmap for now.
pub type Symtab = HashMap<Rc<String>, Rc<String>>;

pub type ParseResult<A> = Result<(A, usize), ParseError>;
enum ParseError {
    EOF,
    Other(String),
}
fn eof<A>() -> ParseResult<A> { Err(ParseError::EOF) }
fn err<A, S: ToString>(s: S) -> ParseResult<A> {
    Err(ParseError::Other(s.to_string()))
}

// regexes needed for parsing.
struct Regexes { ws: Regex, symbol: Regex, string: Regex, number: Regex }

fn parse(syms: &mut Symtab, input: &str) -> ParseResult<Sexp> {
    let re = Regexes {
        ws: Regex::new(r"^\s+").unwrap(),
        symbol: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*").unwrap(),
        // TODO: string escapes.
        string: Regex::new("^\"[^\"]*\"").unwrap(),
        number: Regex::new(r"^\d+").unwrap(),
    };
    parse_sexp(&re, syms, input)
}

fn parse_sexp(re: &Regexes, syms: &mut Symtab, mut input: &str)
              -> ParseResult<Sexp>
{
    input = &input[re.ws.find(input).unwrap_or((0,0)).1 ..];
    if input.is_empty() {
        eof()
    } else if &input[0..1] == "(" {
        return parse_sexps(re, syms, &input[1..])
            .map(|e| (Sexp::List(e.0), e.1 + 1))
    } else if let Some((_,i)) = re.symbol.find(input) {
        // match syms.get(&input[0..i]) {
        //     Some(s) => return Ok((Sexp::Symbol(s.clone()), i)),
        //     None => {}
        // }
        // let s = Rc::new(input[0..i]);
        // syms.insert(s, &s);
        unimplemented!()
    } else if let Some((_,i)) = re.string.find(input) {
        // FIXME: NEED TO DEAL WITH ESCAPES
        Ok((Sexp::String(Rc::new(String::from(&input[1..i-1]))), i))
    } else if let Some((_,i)) = re.number.find(input) {
        match input[0..i].parse::<i64>() {
            Ok(n) => Ok((Sexp::Int(n), i)),
            Err(e) => err(format!("{}", e))
        }
    } else {
        err(String::from("could not parse"))
    }
}

fn parse_sexps(re: &Regexes, syms: &mut Symtab, input: &str)
               -> ParseResult<Vec<Sexp>>
{
    unimplemented!()
}
