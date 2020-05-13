use std::fmt;
use crate::{
    src::{Loc, Span},
    lex::Token,
    node::SrcNode,
};

#[derive(Debug)]
pub struct Error {
    msg: String,
    spans: Vec<Span>,
    hints: Vec<String>,
}

impl Error {
    #[deprecated]
    pub fn custom(msg: String) -> Self {
        Self {
            msg,
            spans: Vec::new(),
            hints: Vec::new(),
        }
    }

    pub fn in_source<'a>(&'a self, src: &'a str) -> ErrorInSrc<'a> {
        ErrorInSrc {
            error: self,
            src,
        }
    }

    pub fn at(mut self, span: Span) -> Self {
        // TODO: More span information
        self
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        // TODO: Merge errors
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.spans.push(span);
        self
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.hints.push(hint);
        self
    }
}

impl parze::error::Error<char> for Error {
    type Span = Span;
    type Thing = Thing;
    type Context = ();

    fn unexpected_sym(c: &char, span: Span) -> Self {
        Self::custom(format!("Unexpected character '{}'", c))
            .with_span(span)
    }

    fn unexpected_end() -> Self {
        Self::custom(format!("Unexpected end of input"))
    }

    fn expected_end(c: &char, span: Span) -> Self {
        Self::custom(format!("Expected end of input, found '{}'", c))
            .with_span(span)
    }

    fn expected(mut self, thing: Self::Thing) -> Self {
        // TODO: Merge error messages
        self
    }

    fn merge(self, other: Self) -> Self {
        self.merge(other)
    }
}

impl parze::error::Error<SrcNode<Token>> for Error {
    type Span = Span;
    type Thing = Thing;
    type Context = ();

    fn unexpected_sym(sym: &SrcNode<Token>, span: Span) -> Self {
        Self::custom(format!("Unexpected token '{}'", **sym))
            .with_span(sym.span())
            .with_span(span)
    }

    fn unexpected_end() -> Self {
        Self::custom(format!("Unexpected end of input"))
    }

    fn expected_end(sym: &SrcNode<Token>, span: Span) -> Self {
        Self::custom(format!("Expected end of input, found '{}'", **sym))
            .with_span(sym.span())
            .with_span(span)
    }

    fn expected(mut self, thing: Self::Thing) -> Self {
        // TODO: Merge error messages
        self
    }

    fn merge(self, other: Self) -> Self {
        self.merge(other)
    }
}

pub struct ErrorInSrc<'a> {
    error: &'a Error,
    src: &'a str,
}

impl<'a> fmt::Display for ErrorInSrc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let highlight_spans = |f: &mut fmt::Formatter, spans: &[_]| {
            let span_iter = spans
                .iter()
                .chain(self.error.spans.iter());

            if let Some(((start_line, start_col), (end_line, end_col))) = span_iter
                .clone()
                .fold(Span::none(), |a, x| a.union(*x))
                .in_context(self.src)
            {
                writeln!(f, "-> line {}, column {}", start_line + 1, start_col + 1)?;

                let lines = self.src.lines().collect::<Vec<_>>();

                let mut char_pos = 0;
                for (i, line) in lines.iter().enumerate() {
                    if i >= start_line && i <= end_line {
                        let line_span = Span::range(
                            Loc::at(char_pos),
                            Loc::at(char_pos + line.len()),
                        );

                        writeln!(f, "{:>4} | {}", i + 1, line.replace("\t", " "))?;

                        // Underline
                        if span_iter
                            .clone()
                            .any(|r| r.intersects(line_span)) {
                            write!(f, "       ")?;
                            for _ in 0..line.len() {
                                if span_iter.clone().any(|r| r.contains(Loc::at(char_pos))) {
                                    write!(f, "^")?;
                                } else {
                                    write!(f, " ")?;
                                }

                                char_pos += 1;
                            }
                            writeln!(f, "")?;
                            char_pos += 1;
                        } else {
                            char_pos += line.len() + 1;
                        }
                    } else {
                        char_pos += line.len() + 1;
                    }
                }
            }

            Ok(())
        };

        writeln!(f, "Error: {}", self.error.msg)?;
        highlight_spans(f, &[])?;

        for hint in self.error.hints.iter() {
            writeln!(f, "Hint: {}", hint)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Thing {
    Char(char),
    Token(Token),
}

impl From<char> for Thing {
    fn from(c: char) -> Self {
        Thing::Char(c)
    }
}

impl From<Token> for Thing {
    fn from(token: Token) -> Self {
        Thing::Token(token)
    }
}

impl From<SrcNode<Token>> for Thing {
    fn from(token: SrcNode<Token>) -> Self {
        Self::from(token.into_inner())
    }
}

impl fmt::Display for Thing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Thing::Char(c) => write!(f, "'{}'", c),
            Thing::Token(t) => write!(f, "'{}'", t),
        }
    }
}
