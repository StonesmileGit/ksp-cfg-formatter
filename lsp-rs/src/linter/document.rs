use ksp_cfg_formatter::parser::DocItem;

use super::{Diagnostic, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::Document<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        for statement in &self.statements {
            let (mut diagnostics, res) = statement.lint(state);
            items.append(&mut diagnostics);
            // Merge result into this result
            result.top_level_no_op_result |= res.map_or(false, |res| res.top_level_no_op_result);
        }
        (items, Some(result))
    }
}

impl<'a> Lintable for DocItem<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        match self {
            DocItem::Node(n) => n.lint(state),
            DocItem::Comment(c) => c.lint(state),
            DocItem::EmptyLine => (vec![], None),
            DocItem::Error(_e) => (vec![], None),
        }
    }
}
