use super::{ASTPrint, Comment, KeyVal, Node};

/// Enum for the different items that can exist in a document/node
#[derive(Debug)]
pub enum NodeItem<'a> {
    /// A node
    Node(Node<'a>),
    /// A Comment
    Comment(Comment<'a>),
    /// An assignment, Not allowed in top level, checked for in `Document` code
    KeyVal(KeyVal<'a>),
    /// An empty line
    EmptyLine,
}
impl<'a> ASTPrint for NodeItem<'a> {
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
            Self::KeyVal(keyval) => {
                keyval.ast_print(depth, indentation, line_ending, should_collapse)
            }
            Self::EmptyLine => line_ending.to_owned(),
        }
    }
}
