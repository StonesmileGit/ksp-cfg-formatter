mod document;
mod has;
mod key_val;
mod node;

pub(crate) fn lint_ast(
    ast: &ksp_cfg_formatter::parser::Document,
    this_url: url::Url,
) -> Vec<Diagnostic> {
    // Only return the Diagnostc part, and ignore the result at this point
    ast.lint(&LinterState {
        this_url,
        top_level_no_op: None,
    })
    .0
}

#[derive(Clone)]
struct LinterState {
    this_url: url::Url,
    top_level_no_op: Option<Location>,
}

struct LinterStateResult {
    top_level_no_op_result: bool,
}

trait Lintable {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>);
}

pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<Severity>,
    pub message: String,
    pub source: Option<String>,
    pub related_information: Option<Vec<RelatedInformation>>,
}

#[derive(Clone)]
pub struct RelatedInformation {
    pub message: String,
    pub location: Location,
}

#[derive(Clone)]
pub struct Location {
    pub url: url::Url,
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

use ksp_cfg_formatter::parser::{nom::Severity, NodeItem, Range};
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

impl<'a> Lintable for ksp_cfg_formatter::parser::Comment<'a> {
    fn lint(&self, _state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        (vec![], None)
    }
}
