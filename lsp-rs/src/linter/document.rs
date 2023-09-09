use super::{Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::Document<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        for statement in &self.statements {
            let (mut diagnostics, res) = statement.lint(state);
            items.append(&mut diagnostics);
            result.top_level_no_op_result |= res.map_or(false, |res| res.top_level_no_op_result);
        }
        (items, Some(result))
    }
}
