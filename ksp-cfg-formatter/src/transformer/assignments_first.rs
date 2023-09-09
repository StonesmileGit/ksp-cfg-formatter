use crate::parser::{Document, Error, NodeItem};

/// Moves assignments first in the node, and child nodes last
/// # Errors
/// Returns an error if an empty line, or a comment is found at the bottom of a node
// Assume empty line between keys and nodes
pub fn assignments_first(mut doc: Document) -> Result<Document, Error> {
    doc.statements = reorder_node_items(doc.statements)?;
    Ok(doc)
}

fn reorder_node_items(mut node_items: Vec<NodeItem>) -> Result<Vec<NodeItem>, Error> {
    let mut key_stuff = vec![];
    let mut node_stuff = vec![];

    let mut processing_key: Option<bool> = None;
    node_items.reverse();
    for item in node_items {
        match item {
            NodeItem::Node(mut node) => {
                processing_key = Some(false);
                node.block = reorder_node_items(node.block)?;
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
        }
    }
    key_stuff.reverse();
    node_stuff.reverse();
    let mut new_node_items = vec![];
    new_node_items.append(&mut key_stuff);
    new_node_items.append(&mut node_stuff);
    Ok(new_node_items)
}
