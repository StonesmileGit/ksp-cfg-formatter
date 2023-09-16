use itertools::Itertools;
use nom::{
    branch::alt,
    combinator::{eof, map},
    multi::many_till,
};
use pest::iterators::Pair;

use super::{
    nom::{
        utils::{self, debug_fn, expect, ignore_line_ending, ws},
        CSTParse, IResult, LocatedSpan,
    },
    ASTPrint, Comment, Error, Node, Rule,
};

/// Enum for the different items that can exist in a document/node
#[derive(Debug, Clone)]
pub enum DocItem<'a> {
    /// A node
    Node(Node<'a>),
    /// A Comment
    Comment(Comment<'a>),
    /// An empty line
    EmptyLine,
    /// An error instead of a doc item
    Error,
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
            Self::Error => todo!(),
        }
    }
}

/// Contains all the statements of a file
#[derive(Debug, Clone)]
pub struct Document<'a> {
    /// List of all the statements. Can be `Node`s, `Comment`s, or `EmptyLine`s
    pub statements: Vec<DocItem<'a>>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Document<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Error> {
        for statement in rule.clone().into_inner() {
            if statement.as_rule() == Rule::assignment {
                return Err(Error {
                    reason: super::Reason::Custom("Top level assignment found".to_string()),
                    source_text: statement.as_str().to_string(),
                    location: Some(statement.into()),
                });
            }
        }
        let statements = parse_block_items(rule, true)?;
        Ok(Document { statements })
    }
}
fn parse_block_items(pair: Pair<Rule>, top_level: bool) -> Result<Vec<DocItem>, Error> {
    assert!(matches!(pair.as_rule(), Rule::nodeBody | Rule::document));
    let mut block_items = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::node => block_items.push(Ok(DocItem::Node(Node::try_from((pair, top_level))?))),
            Rule::Comment => block_items.push(Ok(DocItem::Comment(
                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
            ))),
            Rule::EmptyLine => block_items.push(Ok(DocItem::EmptyLine)),
            Rule::EOI | Rule::Newline => (),
            _ => unreachable!(),
        }
    }
    block_items.into_iter().collect()
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

impl<'a> CSTParse<'a, Document<'a>> for Document<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Document<'a>> {
        map(
            many_till(
                debug_fn(
                    expect(
                        alt((
                            debug_fn(
                                map(ignore_line_ending(ws(Comment::parse)), DocItem::Comment),
                                "Got Doc Comment",
                                false,
                            ),
                            map(ignore_line_ending(ws(Node::parse)), DocItem::Node),
                            debug_fn(
                                map(utils::empty_line, |_| DocItem::EmptyLine),
                                "Got empty line",
                                false,
                            ),
                        )),
                        "Only Nodes, Comments and Empty lines are allowed on the top level",
                    ),
                    "Got DocItem",
                    true,
                ),
                eof,
            ),
            |inner| {
                let res = Document {
                    statements: inner
                        .0
                        .iter()
                        .map(|a| {
                            a.clone()
                                // .unwrap_or(Some(DocItem::Error))
                                .unwrap_or(DocItem::Error)
                        })
                        .collect_vec(),
                };
                // dbg!(&res);
                res
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_doc() {
        let input = "node { key = val }\r\n";
        let res = Document::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_2() {
        let input = "node\r\n{\r\n\tkey = val\r\n\tkey = val\r\n}\r\n";
        let res = Document::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_doc_3() {
        let input = "//1\r\n\r\n//2\r\n";
        let res = Document::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
}
