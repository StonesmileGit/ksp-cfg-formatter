use super::{
    nom::{
        utils::{range_wrap, ws},
        CSTParse, IResult, LocatedSpan,
    },
    ASTPrint, Ranged,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take},
    combinator::{map, recognize},
    sequence::pair,
};
use std::fmt::Display;

/// A comment in the file. Includes the leading `//`
#[derive(Debug, Clone, Copy)]
pub struct Comment<'a> {
    /// Text of the comment, including the leading `//`
    pub text: &'a str,
}

impl<'a> Display for Comment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<'a> ASTPrint for Comment<'a> {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!("{}{}{}", indentation, self.text, line_ending)
    }
}

impl<'a> CSTParse<'a, Ranged<Comment<'a>>> for Comment<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Comment<'a>>> {
        let comment = recognize(ws(pair(tag("//"), alt((is_not("\r\n"), take(0usize))))));
        range_wrap(map(comment, |inner: LocatedSpan| Comment {
            text: inner.fragment(),
        }))(input)
    }
}
