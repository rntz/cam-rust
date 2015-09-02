#![allow(unused_variables,dead_code,unused_imports)]

use std::collections::{HashSet};
use std::str::Chars;

use regex::Regex;

#[derive(Clone)]
pub enum Sexp<'a> {
    Int(i64),
    String(String),
    Symbol(&'a str),
    List(Vec<Sexp<'a>>),
}

pub type Symtab = HashSet<String>;

struct Regexes {
    ws: Regex,
    symbol: Regex,
    string: Regex,
    number: Regex,
}

enum ParseError {
    EOF,
    Other(String),
}

impl<'a> Sexp<'a> {
    fn parse(syms: &'a mut Symtab, input: &str)
             -> Result<(Sexp<'a>, usize), ParseError> {
        let re = Regexes {
            ws: Regex::new(r"^\s+").unwrap(),
            symbol: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*").unwrap(),
            // TODO: string escapes.
            string: Regex::new("^\"[^\"]*\"").unwrap(),
            number: Regex::new(r"^\d+").unwrap(),
        };
        Sexp::parse_sexp(&re, syms, input)
    }

    fn parse_sexp(re: &Regexes, syms: &'a mut Symtab, mut input: &str)
             -> Result<(Sexp<'a>, usize), ParseError> {
        input = &input[re.ws.find(input).unwrap_or((0,0)).1 ..];
        if input.is_empty() {
            Err(ParseError::EOF)
        } else if &input[0..1] == "(" {
            return Sexp::parse_sexps(re, syms, &input[1..])
                .map(|e| (Sexp::List(e.0), e.1 + 1))
        } else if let Some((_,i)) = re.symbol.find(input) {
            unimplemented!()
        } else if let Some((_,i)) = re.string.find(input) {
            // FIXME: NEED TO DEAL WITH ESCAPES
            Ok((Sexp::String(String::from(&input[1..i-1])), i))
        } else if let Some((_,i)) = re.number.find(input) {
            match input[0..i].parse::<i64>() {
                Ok(n) => Ok((Sexp::Int(n), i)),
                Err(e) => Err(ParseError::Other(format!("{}", e)))
            }
        } else {
            Err(ParseError::Other(String::from("could not parse")))
        }
    }

    fn parse_sexps(re: &Regexes, syms: &'a mut Symtab, mut input: &str)
                   -> Result<(Vec<Sexp<'a>>, usize), ParseError> {
        unimplemented!()
    }
}
