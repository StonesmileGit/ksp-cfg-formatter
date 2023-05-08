use std::fmt::Display;

use pest::iterators::Pairs;
use pest_derive::Parser;

use super::printer::{Comment, KeyVal, Node, NodeItem};

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

fn parse_assignment(mut pairs: Pairs<Rule>) -> NodeItem {
    let key = pairs.next().unwrap();
    let operator = pairs.next().unwrap();
    let value = pairs.next().unwrap();
    let mut val_comment = None;
    if let Some(comment) = pairs.next() {
        if matches!(comment.as_rule(), Rule::Comment) {
            val_comment = Some(Comment {
                text: comment.as_str().to_string(),
            });
        }
    }
    let val_str = if val_comment.is_none() {
        value.as_str().trim().to_string()
    } else {
        value.as_str().to_string()
    };
    NodeItem::KeyVal(KeyVal {
        key: key.as_str().trim().to_string(),
        operator: operator.as_str().trim().to_string(),
        val: val_str,
        comment: val_comment,
    })
}

fn parse_node(mut pairs: Pairs<Rule>) -> NodeItem {
    let identifier = pairs.next().unwrap();
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
            _ => unreachable!(),
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
        identifier: identifier.as_str().trim().to_string(),
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
        }
    }
}
