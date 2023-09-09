use super::{range_to_range, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::Node<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        // This node has an operator, but is in a top level node that does not!
        if state.top_level_no_op.is_some() && self.operator.is_some() {
            items.push(lsp_types::Diagnostic {
                range: range_to_range(
                    self.operator
                        .as_ref()
                        .expect("it was just determined that the operator existed")
                        .get_pos(),
                ),
                severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                message: "Node has operator, even though the top level does not!".to_owned(),
                related_information: Some(vec![lsp_types::DiagnosticRelatedInformation {
                    location: state
                        .top_level_no_op
                        .clone()
                        .expect("It was just determined that the top_level_no_op is Some"),
                    message: "This is where it happened".to_owned(),
                }]),
                code: Some(lsp_types::NumberOrString::Number(1)),
                source: Some("Unexpected_operator".to_owned()),
                ..Default::default()
            });
            result.top_level_no_op_result = true;
        }

        if self.name.clone().map_or(false, |name| name.0.len() > 1) && !self.top_level() {
            items.push(lsp_types::Diagnostic {
                range: range_to_range(self.name.clone().expect("It was just determined that it is Some").1),
                severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                message: "names separated by '|' is only interpreted as OR in a top level node. Here, it's interpreted literally.".to_owned(),
                ..Default::default()
            });
        }

        let mut state: LinterState = state.clone();
        // Check for operators in nodes that do not have any operators
        if self.top_level() && self.operator.is_none() {
            state.top_level_no_op = Some(lsp_types::Location {
                uri: state.this_url.clone(),
                range: range_to_range(self.range),
            });
        }

        for statement in &self.block {
            let (mut diagnostics, res) = statement.lint(&state);
            items.append(&mut diagnostics);
            result.top_level_no_op_result |= res.map_or(false, |res| res.top_level_no_op_result);
        }
        if self.top_level() && result.top_level_no_op_result {
            items.push(lsp_types::Diagnostic {
                range: state
                    .top_level_no_op
                    // .clone()
                    .expect("it was just determined that that top_level_no_op was Some")
                    .range,
                severity: Some(lsp_types::DiagnosticSeverity::HINT),
                message:
                    "This node has no operator, but contains something that does have an operator"
                        .to_owned(),
                ..Default::default()
            });
        }
        (items, Some(result))
    }
}
