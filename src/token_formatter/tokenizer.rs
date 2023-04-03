use logos::Logos;
use std::{fmt::Display, str::FromStr};

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

    /// Token representing any whitespace
    #[regex(r" +|\t+", |lex| lex.slice().parse())]
    Whitespace(Whitespace),

    /// Token representing `=`
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
            Self::Spaces(n) => {
                let spaces = " ".repeat(*n);
                write!(f, "{spaces}")
            }
            Self::Tabs(n) => {
                let tabs = "\t".repeat(*n);
                write!(f, "{tabs}")
            }
        }
    }
}

/// TODO
pub struct ParseWhiteSpaceError;
impl FromStr for Whitespace {
    type Err = ParseWhiteSpaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(' ') {
            Ok(Whitespace::Spaces(s.chars().count()))
        } else if s.contains('\t') {
            Ok(Whitespace::Tabs(s.chars().count()))
        } else {
            Err(ParseWhiteSpaceError)
        }
    }
}
