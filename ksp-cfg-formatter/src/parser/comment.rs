use super::{
    nom::{utils::ws, CSTParse},
    ASTPrint, Range, Rule,
};
use nom::combinator::recognize;
use pest::iterators::Pair;
use std::{convert::Infallible, fmt::Display};

/// A comment in the file. Includes the leading `//`
#[derive(Debug, Clone, Copy)]
pub struct Comment<'a> {
    /// Text of the comment, including the leading `//`
    pub text: &'a str,
    _range: Range,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Comment<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let range = Range::from(&rule);
        Ok(Comment {
            text: rule.as_str(),
            _range: range,
        })
    }
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

impl<'a> CSTParse<'a, Comment<'a>> for Comment<'a> {
    fn parse(input: super::nom::LocatedSpan<'a>) -> super::nom::IResult<Comment<'a>> {
        let comment = recognize(ws(nom::sequence::pair(
            nom::bytes::complete::tag("//"),
            nom::bytes::complete::is_not("\r\n"),
        )));
        nom::combinator::map(comment, |inner: super::nom::LocatedSpan| Comment {
            text: inner.fragment(),
            // FIXME: This needs to change
            _range: Range::default(),
        })(input)
    }
}
