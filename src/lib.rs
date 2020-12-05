#![feature(or_patterns)]

use std::{collections::HashMap, fmt};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InputIndex(usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LogIndex(usize);

pub enum Event {
    Start { name: String, position: InputIndex },
    Commit,
    Abort,
    Consume { token: String },
    Reference { reference: LogIndex },
}

#[derive(Default)]
pub struct Sink {
    log: Vec<Event>,
    memo: HashMap<(String, InputIndex), LogIndex>,
}

impl fmt::Display for Sink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut level = 0;
        for log in self.log.iter() {
            if matches!(log, Event::Commit | Event::Abort) {
                level -= 1;
            }
            for _ in 0..level {
                write!(f, "  ")?;
            }
            match log {
                Event::Start { name, position } => {
                    writeln!(f, "START[{}, {}]", name, position.0)?;
                    level += 1;
                }
                Event::Commit => {
                    writeln!(f, "COMMIT")?;
                }
                Event::Abort => {
                    writeln!(f, "ABORT")?;
                }
                Event::Consume { token } => {
                    writeln!(f, "CONSUME[{}]", token)?;
                }
                Event::Reference { reference } => {
                    writeln!(f, "REFERENCE[{}]", reference.0)?;
                }
            }
        }

        writeln!(f)?;

        for ((name, input), log) in self.memo.iter() {
            writeln!(f, "[{}, {}] -> {}", name, input.0, log.0)?;
        }

        Ok(())
    }
}

impl Sink {
    fn start(&mut self, name: String, position: InputIndex) {
        self.memo
            .insert((name.clone(), position), LogIndex(self.log.len()));
        self.log.push(Event::Start { name, position });
    }

    fn commit(&mut self) {
        self.log.push(Event::Commit);
    }

    fn abort(&mut self) {
        self.log.push(Event::Abort);
    }
}

#[derive(Clone)]
pub struct Parser<'s> {
    input: &'s str,
    position: InputIndex,
}

impl<'s> Parser<'s> {
    pub fn new(input: &'s str) -> Self {
        Self {
            input,
            position: InputIndex(0),
        }
    }

    fn remaining(&self) -> &'s str {
        &self.input[self.position.0..]
    }

    fn eat(&mut self, token: &str, sink: &mut Sink) -> bool {
        if self.remaining().starts_with(token) {
            sink.log.push(Event::Consume {
                token: token.into(),
            });
            self.position = InputIndex(self.position.0 + token.len());
            true
        } else {
            false
        }
    }

    fn parse<P>(&mut self, sink: &mut Sink, parselet: &mut P) -> bool
    where
        P: ParseLet,
    {
        let backtrack = self.clone();
        sink.start(parselet.name(), self.position);
        if parselet.parse(self, sink) {
            sink.commit();
            true
        } else {
            sink.abort();
            *self = backtrack;
            false
        }
    }
}

pub trait ParseLet {
    fn name(&self) -> String;
    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool;
}

// T = F + T | F
// F = P * F | P
// P = 0 | 1 | ... | 9
struct Primitive;
impl ParseLet for Primitive {
    fn name(&self) -> String {
        "prim".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        match parser.remaining().chars().next() {
            Some(c @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                parser.eat(&parser.remaining()[0..c.len_utf8()], sink);
                true
            }
            _ => false,
        }
    }
}

struct Factor1;
impl ParseLet for Factor1 {
    fn name(&self) -> String {
        "factor(1)".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Primitive)
            && parser.eat("*", sink)
            && parser.parse(sink, &mut Factor)
    }
}

struct Factor2;
impl ParseLet for Factor2 {
    fn name(&self) -> String {
        "factor(2)".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Primitive)
    }
}

struct Factor;
impl ParseLet for Factor {
    fn name(&self) -> String {
        "factor".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Factor1) || parser.parse(sink, &mut Factor2)
    }
}

struct Term1;
impl ParseLet for Term1 {
    fn name(&self) -> String {
        "term(1)".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Factor) && parser.eat("+", sink) && parser.parse(sink, &mut Term)
    }
}

struct Term2;
impl ParseLet for Term2 {
    fn name(&self) -> String {
        "term(2)".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Factor)
    }
}

struct Term;
impl ParseLet for Term {
    fn name(&self) -> String {
        "term".into()
    }

    fn parse(&mut self, parser: &mut Parser, sink: &mut Sink) -> bool {
        parser.parse(sink, &mut Term1) || parser.parse(sink, &mut Term2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn math() {
        let mut parser = Parser::new("1*3");
        let mut sink = Sink::default();

        parser.parse(&mut sink, &mut Term);
        println!("{}", sink);
    }
}
