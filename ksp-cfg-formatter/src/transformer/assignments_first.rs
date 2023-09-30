use itertools::Itertools;

use crate::parser::{DocItem, Document, Error, NodeItem};

/// Moves assignments first in the node, and child nodes last
/// # Errors
/// Returns an error if an empty line, or a comment is found at the bottom of a node
// Assume empty line between keys and nodes
pub fn assignments_first(mut doc: Document) -> Result<Document, Error> {
    doc.statements = reorder_doc_items(doc.statements)?;
    Ok(doc)
}

fn reorder_doc_items(items: Vec<DocItem>) -> Result<Vec<DocItem>, Error> {
    let items = items
        .into_iter()
        .map(|e| match e {
            DocItem::Node(n) => NodeItem::Node(n.clone()),
            DocItem::Comment(c) => NodeItem::Comment(c),
            DocItem::EmptyLine => NodeItem::EmptyLine,
            DocItem::Error(e) => NodeItem::Error(e),
        })
        .collect();
    let items = reorder_node_items(items)?;
    Ok(items
        .into_iter()
        .map(|e| match e {
            NodeItem::Node(n) => DocItem::Node(n.clone()),
            NodeItem::Comment(c) => DocItem::Comment(c),
            NodeItem::EmptyLine => DocItem::EmptyLine,
            NodeItem::Error(e) => DocItem::Error(e),
            NodeItem::KeyVal(_) => unreachable!(),
        })
        .collect_vec())
}

fn reorder_node_items<'a>(mut node_items: Vec<NodeItem<'a>>) -> Result<Vec<NodeItem<'a>>, Error> {
    let mut key_stuff = vec![];
    let mut node_stuff = vec![];

    let mut processing_key: Option<bool> = None;
    node_items.reverse();
    for item in node_items {
        match item {
            NodeItem::Node(mut node) => {
                processing_key = Some(false);
                node.block = reorder_node_items(node.block.clone())?;
                node_stuff.push(NodeItem::Node(node));
            }
            NodeItem::Comment(_) => match processing_key {
                Some(true) => key_stuff.push(item),
                Some(false) => node_stuff.push(item),
                None => {
                    return Err(Error {
                        reason: crate::parser::Reason::Custom(
                            "Found Comment at end of node".to_string(),
                        ),
                        location: None,
                        source_text: String::new(),
                    })
                }
            },
            NodeItem::KeyVal(_) => {
                processing_key = Some(true);
                key_stuff.push(item);
            }
            NodeItem::EmptyLine => match processing_key {
                Some(true) => key_stuff.push(item),
                Some(false) => node_stuff.push(item),
                None => {
                    return Err(Error {
                        reason: crate::parser::Reason::Custom(
                            "Found Empty Line at end of node".to_string(),
                        ),
                        location: None,
                        source_text: String::new(),
                    })
                }
            },
            NodeItem::Error(_e) => todo!(),
        }
    }
    key_stuff.reverse();
    node_stuff.reverse();
    let mut new_node_items = vec![];
    new_node_items.append(&mut key_stuff);
    new_node_items.append(&mut node_stuff);
    Ok(new_node_items)
}
