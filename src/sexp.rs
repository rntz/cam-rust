use std::collections::{HashMap};
use std::str::FromStr;

use regex::Regex;

use string::Str;

#[derive(Clone,Debug)]
pub enum Sexp {
    Int(i64),
    String(Str),
    Symbol(Str),
    List(Vec<Sexp>),
}

impl FromStr for Sexp {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Sexp, ParseError> {
        let mut syms = HashMap::new();
        parse(&mut syms, s).and_then(|e| {
            let (sexp, i) = e;
            if s[i..].trim_left().is_empty() {
                Ok(sexp)
            } else {
                Err(ParseError::Other(
                    String::from("leftover noise in string")))
            }
        })
    }
}

// TODO: impl Display for Sexp<'a>

// we can't get keys out of hashsets by looking them up, so we have to use an
// identity hashmap for now.
type Symtab = HashMap<Str, Str>;

type ParseResult<A> = Result<(A, usize), ParseError>;
#[derive(Debug)]
pub enum ParseError {
    EOF,
    RightParen,
    Other(String),
}
fn eof<A>() -> ParseResult<A> { Err(ParseError::EOF) }
fn err<A, S: ToString>(s: S) -> ParseResult<A> {
    Err(ParseError::Other(s.to_string()))
}

// regexes needed for parsing.
struct Regexes { ws: Regex, symbol: Regex, string: Regex, number: Regex }

// first index after i that isn't whitespace
fn skip_ws(re: &Regexes, input: &str, i: usize) -> usize {
    i + re.ws.find(&input[i..]).unwrap_or((0,0)).1
}

fn parse(syms: &mut Symtab, input: &str) -> ParseResult<Sexp> {
    let re = Regexes {
        ws: Regex::new(r"^\s+").unwrap(),
        symbol: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*").unwrap(),
        // TODO: string escapes.
        string: Regex::new("^\"[^\"]*\"").unwrap(),
        number: Regex::new(r"^\d+").unwrap(),
    };
    parse_sexp(&re, syms, input, skip_ws(&re, input, 0))
}

fn parse_sexp(re: &Regexes, syms: &mut Symtab, input: &str, i: usize)
              -> ParseResult<Sexp>
{
    let rest = &input[i..];
    if rest.is_empty() {
        eof()
    } else if &rest[0..1] == "(" {
        parse_sexps(re, syms, input, i+1).and_then(|e| {
            let (v,j) = e;
            if j == input.len() {
                eof()
            } else if ")" == &input[j..j+1] {
                Ok((Sexp::List(v), j+1))
            } else {
                err("unknown")
            }
        })
    } else if &rest[0..1] == ")" {
        Err(ParseError::RightParen)
    } else if let Some((_,j)) = re.symbol.find(rest) {
        match syms.get(&rest[0..j]) {
            Some(s) => return Ok((Sexp::Symbol(s.clone()), i+j)),
            None => {}
        }
        let s = Str::new(&rest[0..j]);
        let (sk,sv) = (s.clone(), s.clone());
        syms.insert(sk, sv);
        Ok((Sexp::Symbol(s), i+j))
    } else if let Some((_,j)) = re.string.find(rest) {
        // FIXME: NEED TO DEAL WITH ESCAPES
        Ok((Sexp::String(Str::new(&rest[1..j-1])), i+j))
    } else if let Some((_,j)) = re.number.find(rest) {
        match rest[0..j].parse::<i64>() {
            Ok(n) => Ok((Sexp::Int(n), i+j)),
            Err(e) => err(format!("{}", e))
        }
    } else {
        err(String::from("could not parse"))
    }
}

fn parse_sexps(re: &Regexes, syms: &mut Symtab, input: &str, mut i: usize)
               -> ParseResult<Vec<Sexp>>
{
    let mut v = vec![];
    loop {
        i = skip_ws(re, input, i);
        match parse_sexp(re, syms, input, i) {
            Ok((e, j)) => { v.push(e); i = j; }
            Err(ParseError::RightParen) | Err(ParseError::EOF)
                => return Ok((v, i)),
            Err(e) => return Err(e)
        }
    }
}
