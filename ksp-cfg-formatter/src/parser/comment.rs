use super::{
    parser_helpers::{range_wrap, ws},
    ASTPrint, Ranged, {ASTParse, IResult, LocatedSpan},
};
use nom::{
    bytes::complete::{is_not, tag},
    combinator::{map, opt, recognize},
    sequence::pair,
};

/// A comment in the file. Includes the leading whitespace and `//`
#[derive(Debug, Clone, Copy)]
pub struct Comment<'a> {
    /// Text of the comment, including leading whitespace and `//`
    pub text: &'a str,
}

impl<'a> ASTPrint for Comment<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        _: Option<bool>,
    ) -> String {
        format!("{}{}{}", indentation.repeat(depth), self.text, line_ending)
    }
}

impl<'a> ASTParse<'a> for Comment<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Comment<'a>>> {
        let comment = recognize(ws(pair(tag("//"), opt(is_not("\r\n")))));

        range_wrap(map(comment, |inner: LocatedSpan| Comment {
            text: inner.fragment(),
        }))(input)
    }
}
