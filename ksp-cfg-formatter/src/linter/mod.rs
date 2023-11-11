mod document;
mod has;
mod key_val;
mod node;

/// Takes a `Document` and lints the AST
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

/// Struct for a diagnostic message, like an error or warning
#[derive(Debug)]
pub struct Diagnostic {
    /// The text range the diagnostic covers
    pub range: Range,
    /// The severity of the diagnostic
    pub severity: Option<Severity>,
    /// The message provided as an explanation for the diagnostic
    pub message: String,
    /// The source text causing the diagnostic
    pub source: Option<String>,
    /// Any related information to the diagnostic, if applicable
    pub related_information: Option<Vec<RelatedInformation>>,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.range, self.message)
    }
}

/// Information relating to another diagnostic
#[derive(Clone, Debug)]
pub struct RelatedInformation {
    /// The message provided for the related info
    pub message: String,
    /// The location of the related info
    pub location: Location,
}

/// A location in a (optional) file
#[derive(Clone, Debug)]
pub struct Location {
    /// An optional Url to the file
    pub url: Option<url::Url>,
    /// The range of the location
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
