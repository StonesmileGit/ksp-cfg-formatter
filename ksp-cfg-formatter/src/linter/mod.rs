mod document;
mod has;
mod key_val;
mod node;

pub fn lint_ast(ast: &crate::parser::Document, this_url: Option<url::Url>) -> Vec<Diagnostic> {
    // Only return the Diagnostic part, and ignore the result at this point
    ast.lint(&LinterState {
        this_url,
        top_level_no_op: None,
    })
    .0
}

#[derive(Clone)]
struct LinterState {
    this_url: Option<url::Url>,
    top_level_no_op: Option<Location>,
}

struct LinterStateResult {
    top_level_no_op_result: bool,
}

trait Lintable {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>);
}

#[derive(Debug)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<Severity>,
    pub message: String,
    pub source: Option<String>,
    pub related_information: Option<Vec<RelatedInformation>>,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.range, self.message)
    }
}

#[derive(Clone, Debug)]
pub struct RelatedInformation {
    pub message: String,
    pub location: Location,
}

#[derive(Clone, Debug)]
pub struct Location {
    pub url: Option<url::Url>,
    pub range: Range,
}

impl Default for Diagnostic {
    fn default() -> Self {
        Self {
            range: Default::default(),
            severity: Default::default(),
            message: Default::default(),
            source: Default::default(),
            related_information: Default::default(),
        }
    }
}

use std::fmt::Display;

use crate::parser::{nom::Severity, NodeItem, Range};
impl<'a> Lintable for NodeItem<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        match self {
            NodeItem::Node(node) => node.lint(state),
            NodeItem::Comment(comment) => comment.lint(state),
            NodeItem::KeyVal(key_val) => key_val.lint(state),
            NodeItem::EmptyLine => (vec![], None),
            NodeItem::Error(_e) => (vec![], None),
        }
    }
}

impl<'a> Lintable for crate::parser::Comment<'a> {
    fn lint(&self, _state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        (vec![], None)
    }
}
