use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take},
    character::complete::{anychar, multispace0},
    combinator::{eof, map, not, opt, recognize, rest},
    multi::many_till,
    sequence::{preceded, terminated, tuple},
};

use super::{
    nom::{
        utils::{self, debug_fn, error_till, expect, ignore_line_ending, non_empty, ws},
        CSTParse, IResult, LocatedSpan,
    },
    ASTPrint, Comment, Node, Ranged,
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
        should_collapse: bool,
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
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for item in &self.statements {
            output.push_str(&item.ast_print(depth, indentation, line_ending, should_collapse));
        }
        output
    }
}

pub fn source_file(input: LocatedSpan) -> IResult<Document> {
    // parse the document, or nothing if that fails
    let doc = alt((
        Document::parse,
        map(take(0usize), |_| Document { statements: vec![] }),
    ));
    // Emitt an error if the whole input is not consumed
    terminated(doc, preceded(expect(not(anychar), "expected EOF"), rest))(input)
}

impl<'a> CSTParse<'a, Document<'a>> for Document<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Document<'a>> {
        map(
            preceded(
                tuple((opt(tag("\u{feff}")), multispace0)),
                many_till(
                    debug_fn(
                        alt((
                            map(ignore_line_ending(ws(Comment::parse)), DocItem::Comment),
                            map(utils::empty_line, |_| DocItem::EmptyLine),
                            map(ignore_line_ending(ws(Node::parse)), DocItem::Node),
                            // If none of the above succeeded, consume the line as an error and try again
                            debug_fn(
                                map(recognize(error_till(non_empty(is_not("}\r\n")))), |a| {
                                    DocItem::Error(Ranged::new(a.clone().fragment(), a.into()))
                                }),
                                "Got an error while parsing doc. Skipped line",
                                true,
                            ),
                        )),
                        "Got DocItem",
                        false,
                    ),
                    eof,
                ),
            ),
            |inner| Document {
                statements: inner.0,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_doc() {
        let input = "node { key = val }\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_2() {
        let input = "node\r\n{\r\n\tkey = val\r\n\tkey = val\r\n}\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_3() {
        let input = "//1\r\n\r\n//2\r\n";
        let res = Document::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
}
