use super::{ASTPrint, Comment, KeyVal, Node, Ranged};

/// Enum for the different items that can exist in a document/node
#[derive(Debug, Clone)]
pub enum NodeItem<'a> {
    /// A node
    Node(Ranged<Node<'a>>),
    /// A Comment
    Comment(Ranged<Comment<'a>>),
    /// An assignment, Not allowed in top level, checked for in `Document` code
    KeyVal(Ranged<KeyVal<'a>>),
    /// An empty line
    EmptyLine,
    /// An error instead of the node item
    Error(Ranged<&'a str>),
}
impl<'a> ASTPrint for NodeItem<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: Option<bool>,
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
            Self::Error(e) => e.to_string(),
        }
    }
}
