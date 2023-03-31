use logos::{Lexer, Logos};
use std::fmt::Display;

/// This enum represents tokens
#[allow(clippy::upper_case_acronyms)]
#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    /// Represents a comment. Includes the leading `//`
    #[regex(r"//[^\r\n]*")]
    Comment(&'a str),

    /// Token representing a newline
    #[token("\n")]
    NewLine,

    /// Token representing a newline
    #[token("\r\n")]
    CRLF,

    /// Token representing an opening bracket, `{`
    #[token(r"{")]
    OpeningBracket,

    /// Token representing a closing bracket, `}`
    #[token(r"}")]
    ClosingBracket,

    /// Token representing any whitespace Currently doesn't care about length
    #[regex(r" +|\t+", parse_whitespace)]
    Whitespace(Whitespace),

    /// Token representing =
    #[token(r"=")]
    Equals,

    /// Token representing any text
    #[regex(r"[^{} \t\r\n]+")]
    Text(&'a str),

    /// Token representing any error
    #[error]
    Error,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Comment(comment) => write!(f, "{comment}"),
            Token::OpeningBracket => write!(f, "{{"),
            Token::ClosingBracket => write!(f, "}}"),
            Token::Whitespace(space) => write!(f, "{space}"),
            Token::Text(text) => write!(f, "{text}"),
            Token::Error => todo!(),
            Token::NewLine => writeln!(f),
            Token::Equals => write!(f, "="),
            Token::CRLF => writeln!(f, "\r"),
        }
    }
}

/// enum representing Whitespace, either spaces or tabs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Whitespace {
    /// `usize` number of spaces
    Spaces(usize),
    /// `usize` number of tabs
    Tabs(usize),
}

impl Display for Whitespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Whitespace::Spaces(n) => {
                let spaces = " ".repeat(*n);
                write!(f, "{spaces}")
            }
            Whitespace::Tabs(n) => {
                let tabs = "\t".repeat(*n);
                write!(f, "{tabs}")
            }
        }
    }
}

fn parse_whitespace<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Whitespace {
    let slice = lex.slice();
    if slice.contains(' ') {
        Whitespace::Spaces(slice.chars().count())
    } else {
        Whitespace::Tabs(slice.chars().count())
    }
}
