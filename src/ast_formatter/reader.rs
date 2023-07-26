use std::{fmt::Display, str::FromStr};

use pest::iterators::Pairs;
use pest_derive::Parser;

use super::printer::{comment::Comment, key_val::KeyVal, operator::Operator, Node, NodeItem};

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

// TODO: Rewrite to check what the next rule is before consuming it
fn parse_assignment(pairs: Pairs<Rule>) -> NodeItem {
    let mut operator = None;
    let mut key = String::new();
    let mut needs = None;
    let mut index = None;
    let mut array_index = None;
    let mut assignment_operator = super::printer::assignment_operator::AssignmentOperator::Assign;
    let mut val = String::new();
    let mut comment = None;
    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => todo!(),
            Rule::document => todo!(),
            Rule::statement => todo!(),
            Rule::openingbracket => todo!(),
            Rule::closingbracket => todo!(),
            Rule::node => todo!(),
            Rule::nodeBeforeBlock => todo!(),
            Rule::nodeBlock => todo!(),
            Rule::assignment => todo!(),
            Rule::identifier => todo!(),
            Rule::value => val = pair.as_str().to_string(),
            Rule::Comment => {
                comment = Some(Comment {
                    text: pair.as_str().to_string(),
                });
            }
            Rule::Whitespace => todo!(),
            Rule::EmptyLine => todo!(),
            Rule::Newline => todo!(),
            Rule::assignmentOperator => {
                assignment_operator =
                    super::printer::assignment_operator::AssignmentOperator::from_str(
                        pair.as_str(),
                    )
                    .ok()
                    .unwrap();
            }
            Rule::nameBlock => todo!(),
            Rule::blocks => todo!(),
            Rule::hasBranch => todo!(),
            Rule::needsBranch => todo!(),
            Rule::passBranch => todo!(),
            Rule::hasBlock => todo!(),
            Rule::needsBlock => needs = Some(pair.as_str().to_string()),
            Rule::modOrClause => todo!(),
            Rule::passBlock => todo!(),
            Rule::firstPassBlock => todo!(),
            Rule::namedPassBlock => todo!(),
            Rule::modName => todo!(),
            Rule::finalPassBlock => todo!(),
            // TODO: Replace with an enum
            Rule::index => index = Some(pair.as_str().to_string()),
            // TODO: Replace with a struct
            Rule::arrayIndex => array_index = Some(pair.as_str().to_string()),
            Rule::operator => {
                operator = Some(
                    Operator::from_str(pair.as_str())
                        .unwrap_or_else(|_| panic!("{}", pair.as_str().to_string())),
                );
            }
            Rule::hasBlockPart => todo!(),
            Rule::hasNode => todo!(),
            Rule::hasKey => todo!(),
            Rule::hasValue => todo!(),
            Rule::keyIdentifier => key = pair.as_str().trim().to_string(),
        }
    }
    NodeItem::KeyVal(KeyVal {
        operator,
        key,
        needs,
        index,
        array_index,
        assignment_operator,
        val: if comment.is_none() {
            val.as_str().trim().to_string()
        } else {
            val
        },
        comment,
    })
}

fn parse_node(mut pairs: Pairs<Rule>) -> NodeItem {
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

            // TODO: Can these be reached? Probably not
            Rule::Whitespace => todo!(),
            Rule::EmptyLine => todo!(),
            Rule::EOI => todo!(),
            Rule::document => todo!(),
            Rule::statement => todo!(),
            Rule::closingbracket => todo!(),
            Rule::node => todo!(),
            Rule::nodeBeforeBlock => todo!(),
            Rule::nodeBlock => todo!(),
            Rule::assignment => todo!(),
            Rule::identifier => identifier = pair.as_str().to_string(),
            Rule::value => todo!(),
            Rule::assignmentOperator => todo!(),
            Rule::nameBlock => name = Some(pair.as_str().to_string()),
            Rule::blocks => todo!(),
            Rule::hasBranch => todo!(),
            Rule::needsBranch => todo!(),
            Rule::passBranch => todo!(),
            Rule::hasBlock => has = Some(pair.as_str().to_string()),
            Rule::needsBlock => needs = Some(pair.as_str().to_string()),
            Rule::modOrClause => todo!(),
            Rule::passBlock => pass = Some(pair.as_str().to_string()),
            Rule::firstPassBlock => todo!(),
            Rule::namedPassBlock => todo!(),
            Rule::modName => todo!(),
            Rule::finalPassBlock => todo!(),
            // TODO: Replace with enum
            Rule::index => index = Some(pair.as_str().to_string()),
            Rule::arrayIndex => todo!(),
            Rule::operator => operator = Some(pair.as_str().to_string()),
            Rule::hasBlockPart => todo!(),
            Rule::hasNode => todo!(),
            Rule::hasKey => todo!(),
            Rule::hasValue => todo!(),
            Rule::keyIdentifier => todo!(),
            // _ => unreachable!(),
        }
    }
    let mut trailing_comment = None;
    let mut rev = pairs.clone().rev();
    let a = rev.next().unwrap();
    if matches!(a.as_rule(), Rule::Comment) {
        trailing_comment = Some(Comment {
            text: a.as_str().to_string(),
        });
    }
    let node = Node {
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
        }
    }
}
