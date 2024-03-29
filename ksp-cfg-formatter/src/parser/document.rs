use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take},
    character::complete::{anychar, multispace0, space1},
    combinator::{eof, map, not, opt, recognize, rest},
    multi::many_till,
    sequence::{pair, preceded, terminated, tuple},
};

use super::{
    parser_helpers::{
        debug_fn, empty_line, error_till, expect, ignore_line_ending, non_empty, range_wrap, ws,
    },
    ASTPrint, Comment, Node, Ranged, {ASTParse, IResult, LocatedSpan},
};

/// Enum for the different items that can exist in a document/node
#[derive(Debug, Clone)]
pub enum DocItem<'a> {
    /// A node
    Node(Ranged<Node<'a>>),
    /// A Comment
    Comment(Ranged<Comment<'a>>),
    /// An empty line
    EmptyLine,
    /// An error instead of a doc item
    Error(Ranged<&'a str>),
}
impl<'a> ASTPrint for DocItem<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: Option<bool>,
    ) -> String {
        match self {
            Self::Node(node) => node.ast_print(depth, indentation, line_ending, should_collapse),
            Self::Comment(comment) => {
                comment.ast_print(depth, indentation, line_ending, should_collapse)
            }
            Self::EmptyLine => line_ending.to_owned(),
            Self::Error(a) => a.to_string(),
        }
    }
}

/// Contains all the statements of a file
#[derive(Debug, Clone)]
pub struct Document<'a> {
    /// List of all the statements. Can be `Node`s, `Comment`s, or `EmptyLine`s
    pub statements: Vec<DocItem<'a>>,
}

impl<'a> ASTPrint for Document<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: Option<bool>,
    ) -> String {
        let mut output = String::new();
        for item in &self.statements {
            output.push_str(&item.ast_print(depth, indentation, line_ending, should_collapse));
        }
        output
    }
}

pub fn source_file(input: LocatedSpan) -> IResult<Ranged<Document>> {
    // parse the document, or nothing if that fails
    let doc = alt((
        Document::parse,
        map(take(0usize), |_| {
            Ranged::new(Document { statements: vec![] }, super::Range::default())
        }),
    ));
    // Emitt an error if the whole input is not consumed
    terminated(doc, preceded(expect(not(anychar), "expected EOF"), rest))(input)
}

impl<'a> ASTParse<'a> for Document<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Document<'a>>> {
        range_wrap(map(
            preceded(
                tuple((opt(tag("\u{feff}")), multispace0)),
                many_till(
                    alt((
                        map(ignore_line_ending(ws(Comment::parse)), DocItem::Comment),
                        map(alt((empty_line, map(pair(space1, eof), |_| ()))), |()| {
                            DocItem::EmptyLine
                        }),
                        map(ignore_line_ending(ws(Node::parse)), DocItem::Node),
                        // If none of the above succeeded, consume the line as an error and try again
                        debug_fn(
                            map(recognize(error_till(non_empty(is_not("\r\n")))), |error| {
                                DocItem::Error(error.into())
                            }),
                            "Got an error while parsing doc. Skipped line",
                            true,
                        ),
                    )),
                    eof,
                ),
            ),
            |inner| Document {
                statements: inner.0,
            },
        ))(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_doc() {
        let input = "node { key = val }\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", Some(true))),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_2() {
        let input = "node\r\n{\r\n\tkey = val\r\n\tkey = val\r\n}\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", Some(true))),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_3() {
        let input = "//1\r\n\r\n//2\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", Some(true))),
            Err(err) => panic!("{}", err),
        }
    }
}
