use crate::parser::{DocItem, Document, Error, Node, NodeItem, Ranged};

/// Moves assignments first in the node, and child nodes last
/// # Errors
/// Returns an error if an empty line, or a comment is found at the bottom of a node
// Assume empty line between keys and nodes
pub fn assignments_first(mut doc: Document) -> Result<Document, Error> {
    let items = doc.statements;
    let mut new_items = vec![];
    for item in items {
        new_items.push(if let DocItem::Node(node) = item {
            let node = reorder_node_items(node)?;
            DocItem::Node(node)
        } else {
            item
        });
    }
    doc.statements = new_items;
    Ok(doc)
}

fn reorder_node_items(mut node: Ranged<Node>) -> Result<Ranged<Node>, Error> {
    let mut node_items = node.block.clone();
    let mut key_stuff = vec![];
    let mut node_stuff = vec![];

    let mut processing_key: Option<bool> = None;
    node_items.reverse();
    for item in node_items {
        match item {
            NodeItem::Node(node) => {
                processing_key = Some(false);
                node_stuff.push(NodeItem::Node(reorder_node_items(node)?));
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
                        severity: crate::parser::nom::Severity::Info,
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
                        severity: crate::parser::nom::Severity::Info,
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
    node.block = new_node_items;
    Ok(node)
}
