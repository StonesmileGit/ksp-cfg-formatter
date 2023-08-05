use super::{ASTPrint, Comment, KeyVal, Node};

#[derive(Debug)]
pub enum NodeItem<'a> {
    Node(Node<'a>),
    Comment(Comment<'a>),
    KeyVal(KeyVal<'a>),
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
