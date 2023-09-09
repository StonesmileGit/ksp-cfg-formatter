mod document;
mod key_val;
mod node;

pub(crate) fn lint_ast(
    ast: &ksp_cfg_formatter::parser::Document,
    this_url: lsp_types::Url,
) -> Vec<lsp_types::Diagnostic> {
    // Only return the Diagnostc part, and ignore the result at this point
    ast.lint(&LinterState {
        this_url,
        top_level_no_op: None,
    })
    .0
}

#[derive(Clone)]
struct LinterState {
    this_url: lsp_types::Url,
    top_level_no_op: Option<lsp_types::Location>,
}

struct LinterStateResult {
    top_level_no_op_result: bool,
}

trait Lintable {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>);
}

// Notice the ´-1´s to get correct 0-indexed position in VSCode
pub(crate) fn range_to_range(parser_range: ksp_cfg_formatter::parser::Range) -> lsp_types::Range {
    lsp_types::Range::new(
        lsp_types::Position::new(parser_range.start.line - 1, parser_range.start.char - 1),
        lsp_types::Position::new(parser_range.end.line - 1, parser_range.end.char - 1),
    )
}

use ksp_cfg_formatter::parser::NodeItem as NI;
impl<'a> Lintable for NI<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        match self {
            NI::Node(node) => node.lint(state),
            NI::Comment(comment) => comment.lint(state),
            NI::KeyVal(key_val) => key_val.lint(state),
            NI::EmptyLine => (vec![], None),
        }
    }
}

impl<'a> Lintable for ksp_cfg_formatter::parser::Comment<'a> {
    fn lint(
        &self,
        _state: &LinterState,
    ) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        // TODO
        (vec![], None)
    }
}