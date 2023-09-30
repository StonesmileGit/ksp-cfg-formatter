use itertools::Itertools;
use strsim::normalized_levenshtein;

use crate::parser::{DocItem, Document, KeyVal, NodeItem, Ranged};

/// Returns `None` if the strings are not similair enough, otherwise the max length is returned.
fn max_len_if_similar(a: &str, b: &str) -> Option<usize> {
    const MIN_CLOSENESS: f64 = 0.8;
    const MAX_DIFF: usize = 4;

    if normalized_levenshtein(a, b) < MIN_CLOSENESS {
        return None;
    };
    let a_len: usize = a.chars().count();
    let b_len: usize = b.chars().count();
    if a_len.abs_diff(b_len) > MAX_DIFF {
        return None;
    };
    Some(a_len.max(b_len))
}

fn max_len_in_vec_if_similar(strs: &[Ranged<KeyVal>]) -> Option<usize> {
    strs.iter()
        .map(|e| e.left_side())
        .tuple_windows()
        .map(|t: (String, String)| max_len_if_similar(t.0.as_str(), t.1.as_str()))
        .reduce(|a, b| {
            if a.is_none() | b.is_none() {
                None
            } else {
                a.max(b)
            }
        })
        .unwrap_or(None)
}

/// pads any assignments where similar keys are found in the immediately adjacent lines, with no empty lines in between
#[must_use]
pub fn assignment_padding(mut doc: Document) -> Document {
    doc.statements = handle_doc_items(doc.statements);
    doc
}

fn handle_doc_items(items: Vec<DocItem>) -> Vec<DocItem> {
    let items = items
        .into_iter()
        .map(|e| match e {
            DocItem::Node(n) => NodeItem::Node(n.clone()),
            DocItem::Comment(c) => NodeItem::Comment(c),
            DocItem::EmptyLine => NodeItem::EmptyLine,
            DocItem::Error(e) => NodeItem::Error(e),
        })
        .collect_vec();
    let items = handle_node_items(items);
    items
        .into_iter()
        .map(|e| match e {
            NodeItem::Node(n) => DocItem::Node(n.clone()),
            NodeItem::Comment(c) => DocItem::Comment(c),
            NodeItem::EmptyLine => DocItem::EmptyLine,
            NodeItem::Error(e) => DocItem::Error(e),
            NodeItem::KeyVal(_) => unreachable!(),
        })
        .collect_vec()
}

fn handle_node_items<'a>(items: Vec<NodeItem<'a>>) -> Vec<NodeItem<'a>> {
    let mut accumulator: Vec<Ranged<KeyVal>> = vec![];
    let mut processed: Vec<NodeItem> = vec![];
    for item in items {
        match item {
            NodeItem::Node(mut node) => {
                processed = fix_kvs(accumulator, processed);
                accumulator = Vec::new();
                node.block = handle_node_items(node.block.clone());
                processed.push(NodeItem::Node(node));
            }
            NodeItem::Comment(comment) => {
                processed = fix_kvs(accumulator, processed);
                accumulator = Vec::new();
                processed.push(NodeItem::Comment(comment));
            }
            NodeItem::KeyVal(kv) => accumulator.push(kv),
            NodeItem::EmptyLine => {
                processed = fix_kvs(accumulator, processed);
                accumulator = Vec::new();
                processed.push(NodeItem::EmptyLine);
            }
            NodeItem::Error(_e) => todo!(),
        }
    }
    fix_kvs(accumulator, processed)
}

fn fix_kvs<'a>(
    accumulator: Vec<Ranged<KeyVal<'a>>>,
    mut processed: Vec<NodeItem<'a>>,
) -> Vec<NodeItem<'a>> {
    // TODO: If accumulator is almost empty, is it worth aligning then?
    let padded_len = max_len_in_vec_if_similar(&accumulator);
    if let Some(padded_len) = padded_len {
        for mut kv in accumulator {
            kv.set_key_padding(padded_len);
            processed.push(NodeItem::KeyVal(kv));
        }
    } else {
        for kv in accumulator {
            processed.push(NodeItem::KeyVal(kv));
        }
    }
    processed
}

// #[cfg(test)]
// mod tests {
//     use super::max_len_in_vec_if_similar;

//     #[test]
//     fn vec_option_to_option_vec_none() {
//         let input = vec!["test".to_owned(), "abcd".to_owned()];
//         let res = max_len_in_vec_if_similar(input.iter());
//         assert_eq!(res, None);
//     }
//     #[test]
//     fn vec_option_to_option_vec_some() {
//         let input = vec!["test".to_owned(), "test".to_owned(), "testa".to_owned()];
//         let res = max_len_in_vec_if_similar(input.iter());
//         assert_eq!(res, Some(5));
//     }
// }
