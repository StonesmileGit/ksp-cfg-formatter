use pest::iterators::Pairs;
use pest_derive::Parser;

use super::printer::{Comment, KeyVal, Node, NodeItem};

pub fn parse_statements(pairs: Pairs<Rule>) -> Vec<NodeItem> {
    let mut doc_items = vec![];
    for pair in pairs {
        match pair.as_rule() {
            Rule::node => doc_items.push(parse_node(pair.into_inner())),
            Rule::Comment => doc_items.push(NodeItem::Comment(Comment {
                text: pair.as_str().to_string(),
            })),
            Rule::assignment => doc_items.push(parse_assignment(pair.into_inner())),
            Rule::EmptyLine => doc_items.push(NodeItem::EmptyLine),
            Rule::closingbracket => break,
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    doc_items
}

fn parse_assignment(mut pairs: Pairs<Rule>) -> NodeItem {
    let first = pairs.next().unwrap();
    let second = pairs.next().unwrap();
    let mut val_comment = None;
    if let Some(comment) = pairs.next() {
        if matches!(comment.as_rule(), Rule::Comment) {
            val_comment = Some(Comment {
                text: comment.as_str().to_string(),
            });
        }
    }
    NodeItem::KeyVal(KeyVal {
        key: first.as_str().to_string(),
        val: second.as_str().to_string(),
        comment: val_comment,
    })
}

fn parse_node(mut pairs: Pairs<Rule>) -> NodeItem {
    let first = pairs.next().unwrap();
    let mut comment = None;
    for a in pairs.by_ref() {
        match a.as_rule() {
            Rule::EOI => todo!(),
            Rule::document => todo!(),
            Rule::statement => todo!(),
            Rule::node => todo!(),
            Rule::assignment => todo!(),
            Rule::identifier => todo!(),
            Rule::value => todo!(),
            Rule::Comment => {
                comment = Some(Comment {
                    text: a.as_str().to_string(),
                });
            }
            Rule::WHITESPACE => todo!(),
            Rule::openingbracket => break,
            Rule::closingbracket => todo!(),
            Rule::EmptyLine => todo!(),
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
        identifier: first.as_str().to_string(),
        id_comment: comment,
        block: parse_statements(pairs),
        trailing_comment,
    };
    NodeItem::Node(node)
}

#[derive(Parser)]
#[grammar = "ast_formatter/grammar.pest"]
pub struct Grammar;
