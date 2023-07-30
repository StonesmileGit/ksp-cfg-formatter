use std::{fmt::Display, str::FromStr};

use pest::iterators::Pairs;
use pest_derive::Parser;

use super::printer::{comment::Comment, key_val::KeyVal, node::Node, operator::Operator, NodeItem};

pub fn parse_block_items(pairs: Pairs<Rule>) -> Vec<NodeItem> {
    let mut block_items = vec![];
    for pair in pairs {
        match pair.as_rule() {
            Rule::node => block_items.push(parse_node(pair.into_inner())),
            Rule::Comment => block_items.push(NodeItem::Comment(Comment {
                text: pair.as_str().to_string(),
            })),
            Rule::assignment => block_items.push(parse_assignment(pair.into_inner())),
            Rule::EmptyLine => block_items.push(NodeItem::EmptyLine),
            Rule::closingbracket => break,
            Rule::EOI | Rule::Newline => (),
            _ => unreachable!(),
        }
    }
    block_items
}

fn parse_assignment(pairs: Pairs<Rule>) -> NodeItem {
    let mut key_val = KeyVal::default();
    for pair in pairs {
        match pair.as_rule() {
            Rule::value => key_val.val = pair.as_str().to_string(),
            Rule::Comment => {
                key_val.comment = Some(Comment {
                    text: pair.as_str().to_string(),
                });
            }
            Rule::assignmentOperator => {
                key_val.assignment_operator =
                    super::printer::assignment_operator::AssignmentOperator::from_str(
                        pair.as_str(),
                    )
                    .ok()
                    .unwrap();
            }
            Rule::needsBlock => key_val.needs = Some(pair.as_str().to_string()),
            Rule::index => {
                key_val.index = Some(
                    super::printer::indices::Index::from_str(pair.as_str())
                        .unwrap_or_else(|_| panic!("{}", pair.as_str().to_string())),
                )
            }
            Rule::arrayIndex => {
                key_val.array_index = Some(
                    super::printer::indices::ArrayIndex::from_str(pair.as_str())
                        .unwrap_or_else(|_| panic!("{}", pair.as_str().to_string())),
                )
            }
            Rule::operator => {
                key_val.operator = Some(
                    Operator::from_str(pair.as_str())
                        .unwrap_or_else(|_| panic!("{}", pair.as_str().to_string())),
                );
            }
            Rule::keyIdentifier => key_val.key = pair.as_str().trim().to_string(),
            Rule::path => key_val.path = Some(pair.as_str().to_string()),
            _ => unreachable!(),
        }
    }
    if key_val.comment.is_none() {
        key_val.val = key_val.val.trim().to_string();
    }
    NodeItem::KeyVal(key_val)
}

fn parse_node(mut pairs: Pairs<Rule>) -> NodeItem {
    let mut path = None;
    let mut operator = None;
    let mut identifier = String::new();
    let mut name = None;
    let mut pass = None;
    let mut has = None;
    let mut needs = None;
    let mut index = None;
    let mut comment = None;
    let mut newline_seen = false;
    let mut comments_after_newline = vec![];
    for pair in pairs.by_ref() {
        match pair.as_rule() {
            Rule::Comment => {
                if newline_seen {
                    comments_after_newline.push(Comment {
                        text: pair.as_str().to_string(),
                    });
                } else {
                    comment = Some(Comment {
                        text: pair.as_str().to_string(),
                    });
                }
            }
            Rule::openingbracket => break,
            Rule::Newline => newline_seen = true,

            Rule::identifier => identifier = pair.as_str().to_string(),
            Rule::nameBlock => name = Some(pair.as_str().to_string()),
            Rule::hasBlock => has = Some(pair.as_str().to_string()),
            Rule::needsBlock => needs = Some(pair.as_str().to_string()),
            Rule::passBlock => pass = Some(pair.as_str().to_string()),
            Rule::index => {
                index = Some(
                    super::printer::indices::Index::from_str(pair.as_str())
                        .unwrap_or_else(|_| panic!("{}", pair.as_str().to_string())),
                )
            }
            Rule::operator => operator = Some(pair.as_str().to_string()),
            Rule::path => path = Some(pair.as_str().to_string()),
            _ => unreachable!(),
        }
    }
    let mut trailing_comment = None;
    let mut rev = pairs.clone().rev();
    let possible_trailing_comment = rev.next().unwrap();
    if matches!(possible_trailing_comment.as_rule(), Rule::Comment) {
        trailing_comment = Some(Comment {
            text: possible_trailing_comment.as_str().to_string(),
        });
    }
    let node = Node {
        path,
        operator,
        identifier: identifier.as_str().trim().to_string(),
        name,
        has,
        needs,
        pass,
        index,
        id_comment: comment,
        comments_after_newline,
        block: parse_block_items(pairs),
        trailing_comment,
    };
    NodeItem::Node(node)
}

#[derive(Parser)]
#[grammar = "ast_formatter/grammar.pest"]
pub struct Grammar;

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::EOI => write!(f, "\"End Of File\""),
            Rule::document => write!(f, "\"Document\""),
            Rule::statement => write!(f, "\"Statement\""),
            Rule::openingbracket => write!(f, "\"Opening bracket\""),
            Rule::closingbracket => write!(f, "\"Closing bracket\""),
            Rule::node => write!(f, "\"Node\""),
            Rule::assignment => write!(f, "\"Key-Value pair\""),
            Rule::identifier => write!(f, "\"Identifier\""),
            Rule::value => write!(f, "\"Value\""),
            Rule::Comment => write!(f, "\"Comment\""),
            Rule::Whitespace => write!(f, "\"Whitespace\""),
            Rule::EmptyLine => write!(f, "\"Empty Line\""),
            Rule::Newline => write!(f, "\"Newline\""),
            Rule::assignmentOperator => write!(f, "\"Assignment Operator\""),
            Rule::nodeBeforeBlock => todo!(),
            Rule::nodeBlock => todo!(),
            Rule::nameBlock => todo!(),
            Rule::blocks => todo!(),
            Rule::hasBranch => todo!(),
            Rule::needsBranch => todo!(),
            Rule::passBranch => todo!(),
            Rule::hasBlock => todo!(),
            Rule::needsBlock => todo!(),
            Rule::modOrClause => todo!(),
            Rule::passBlock => todo!(),
            Rule::firstPassBlock => todo!(),
            Rule::namedPassBlock => todo!(),
            Rule::modName => todo!(),
            Rule::finalPassBlock => todo!(),
            Rule::index => todo!(),
            Rule::arrayIndex => todo!(),
            Rule::operator => todo!(),
            Rule::hasBlockPart => todo!(),
            Rule::hasNode => todo!(),
            Rule::hasKey => todo!(),
            Rule::hasValue => todo!(),
            Rule::keyIdentifier => todo!(),
            Rule::path => todo!(),
            Rule::path_segment => todo!(),
        }
    }
}
