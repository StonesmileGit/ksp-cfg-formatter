use super::{range_to_range, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::KeyVal<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        if state.top_level_no_op.is_some() && self.operator.is_some() {
            items.push(lsp_types::Diagnostic {
                range: range_to_range(
                    self.operator
                        .as_ref()
                        .expect("it was just determined that the operator existed")
                        .get_pos(),
                ),
                severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                code: Some(lsp_types::NumberOrString::Number(1)),
                code_description: None,
                source: Some("Unexpected_operator".to_owned()),
                message: "Key has operator, even though the top level does not!".to_owned(),
                related_information: None,
                tags: None,
                data: None,
            });
            result.top_level_no_op_result = true;
        }

        (items, Some(result))
    }
}
