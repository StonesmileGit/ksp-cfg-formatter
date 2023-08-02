use std::{fmt::Display, num::ParseIntError};

use pest::iterators::Pair;

use crate::Rule;

use super::{
    comment::Comment,
    has::{HasBlock, HasBlockError},
    indices::Index,
    key_val::{KeyVal, KeyValError},
    needs::{NeedsBlock, NeedsBlockError},
    node_item::NodeItem,
    operator::{Operator, OperatorParseError},
    pass::Pass,
    path::Path,
    ASTPrint,
};

#[derive(Debug, Default)]
pub struct Node<'a> {
    pub path: Option<Path<'a>>,
    pub operator: Option<Operator>,
    pub identifier: &'a str,
    pub name: Option<&'a str>,
    pub has: Option<HasBlock<'a>>,
    pub needs: Option<NeedsBlock<'a>>,
    pub pass: Option<Pass<'a>>,
    pub index: Option<Index>,
    pub id_comment: Option<Comment<'a>>,
    pub comments_after_newline: Vec<Comment<'a>>,
    pub block: Vec<NodeItem<'a>>,
    pub trailing_comment: Option<Comment<'a>>,
}

pub(crate) fn parse_block_items(pair: Pair<Rule>) -> Result<Vec<NodeItem>, NodeParseError> {
    assert!(matches!(pair.as_rule(), Rule::nodeBody | Rule::document));
    let mut block_items = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::node => block_items.push(Ok(NodeItem::Node(Node::try_from(pair)?))),
            Rule::Comment => block_items.push(Ok(NodeItem::Comment(
                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
            ))),
            Rule::assignment => block_items.push(Ok(NodeItem::KeyVal(KeyVal::try_from(pair)?))),
            Rule::EmptyLine => block_items.push(Ok(NodeItem::EmptyLine)),
            Rule::EOI | Rule::Newline => (),
            _ => unreachable!(),
        }
    }
    block_items.into_iter().collect()
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum NodeParseError {
    HasBlock(#[from] HasBlockError),
    NeedsBlock(#[from] NeedsBlockError),
    ParseInt(#[from] ParseIntError),
    OperatorParse(#[from] OperatorParseError),
    KeyVal(#[from] KeyValError),
}

impl Display for NodeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Node<'a> {
    type Error = NodeParseError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        assert!(matches!(rule.as_rule(), Rule::node));
        let mut pairs = rule.clone().into_inner();

        let mut node = Node::default();

        let mut body_seen = false;
        let mut newline_seen = false;

        for pair in pairs.by_ref() {
            match pair.as_rule() {
                Rule::Comment => {
                    if body_seen {
                        node.trailing_comment =
                            Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                    } else if newline_seen {
                        node.comments_after_newline.push(
                            Comment::try_from(pair).expect("Parsing a comment is Infallable"),
                        );
                    } else {
                        node.id_comment =
                            Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                    }
                }
                Rule::openingbracket | Rule::closingbracket => (),
                Rule::Newline => newline_seen = true,

                Rule::identifier => node.identifier = pair.as_str(),
                Rule::nameBlock => node.name = Some(pair.as_str()),
                Rule::hasBlock => node.has = Some(HasBlock::try_from(pair)?),
                Rule::needsBlock => node.needs = Some(NeedsBlock::try_from(pair)?),
                Rule::passBlock => {
                    node.pass = Some(Pass::try_from(pair).expect("Should be Infallable"));
                }
                Rule::index => node.index = Some(super::indices::Index::try_from(pair)?),
                Rule::operator => node.operator = Some(Operator::try_from(pair)?),
                Rule::path => {
                    node.path =
                        Some(Path::try_from(pair).expect("Parsing path is supposedly Infallable"));
                }
                Rule::nodeBody => {
                    node.block = parse_block_items(pair)?;
                    body_seen = true;
                }
                _ => unreachable!(),
            }
        }
        Ok(node)
    }
}

impl<'a> ASTPrint for Node<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for comment in &self.comments_after_newline {
            output.push_str(
                comment
                    .ast_print(depth, indentation, line_ending, should_collapse)
                    .as_str(),
            );
        }
        let indentation_str = indentation.repeat(depth);
        let complete_node_name = format!(
            "{}{}{}{}{}{}{}{}{}",
            if self.path.is_some() { "#" } else { "" },
            self.path.clone().map_or(String::new(), |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.identifier,
            self.name.unwrap_or_default(),
            self.has.clone().unwrap_or_default(),
            self.pass.unwrap_or_default(),
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index.map_or(String::new(), |i| i.to_string()),
        );
        output.push_str(
            match self.block.len() {
                0 if self.id_comment.is_none() => {
                    format!(
                        "{}{} {{}}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                        line_ending
                    )
                }
                1 if should_collapse && short_node(self) => {
                    format!(
                        "{}{} {{ {} }}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.block
                            .first()
                            .unwrap()
                            .ast_print(0, indentation, "", should_collapse),
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                        line_ending
                    )
                }
                _ => {
                    let mut output = format!(
                        "{}{}{}{}{}{{{}",
                        indentation_str,
                        complete_node_name,
                        self.id_comment.as_ref().map_or_else(|| "", |c| c.text),
                        line_ending,
                        indentation_str,
                        line_ending
                    );
                    for statement in &self.block {
                        output.push_str(
                            statement
                                .ast_print(depth + 1, indentation, line_ending, should_collapse)
                                .as_str(),
                        );
                    }
                    output.push_str(&indentation_str);
                    output.push('}');
                    output.push_str(
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                    );
                    output.push_str(line_ending);
                    output
                }
            }
            .as_str(),
        );
        output
    }
}

fn short_node(arg: &Node) -> bool {
    const MAX_LENGTH: usize = 72;
    if arg.id_comment.is_some() {
        return false;
    }
    let mut len = 7; // Include the opening/closing bracket and spaces around operator
    len += arg.identifier.chars().count();
    if let Some(name) = arg.name {
        len += name.chars().count();
    }
    len += arg
        .has
        .clone()
        .map_or(0, |has| has.to_string().chars().count());

    match arg.block.first().unwrap() {
        NodeItem::KeyVal(kv) => {
            if kv.operator.is_some() {
                len += 1;
            }
            len += kv.key.chars().count();
            len += kv.assignment_operator.to_string().chars().count();
            len += kv.val.chars().count();
            if kv.comment.is_some() {
                return false;
            };
        }
        _ => return false,
    }
    len <= MAX_LENGTH
}
