use super::{range_to_range, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::Ranged<ksp_cfg_formatter::parser::Node<'a>> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };

        // This node has an operator, but is in a top level node that does not!
        if let Some(diag) = op_in_noop(self, state, &mut result) {
            items.push(diag);
        }
        // The node is filtering on names with '|', but that is only allowed on top level nodes
        if let Some(diag) = or_in_child_node(self, state, &mut result) {
            items.push(diag);
        }

        let mut state: LinterState = state.clone();
        // Check for operators in nodes that do not have any operators
        if self.top_level() && self.operator.is_none() {
            state.top_level_no_op = Some(lsp_types::Location {
                uri: state.this_url.clone(),
                range: range_to_range(self.get_pos()),
            });
        }

        if let Some(has) = &self.has {
            let (mut diagnostics, _res) = has.lint(&state);
            items.append(&mut diagnostics);
        }

        for statement in &self.block {
            let (mut diagnostics, res) = statement.lint(&state);
            items.append(&mut diagnostics);
            // take info from linter results and merge into this linter result
            result.top_level_no_op_result |= res.map_or(false, |res| res.top_level_no_op_result);
        }

        // Add hint diagnostics to aid hints found in block statements
        if let Some(diag) = top_level_no_op_hint(self, &state, &result) {
            items.push(diag);
        }

        (items, Some(result))
    }
}

fn top_level_no_op_hint(
    node: &ksp_cfg_formatter::parser::Node<'_>,
    state: &LinterState,
    result: &LinterStateResult,
) -> Option<lsp_types::Diagnostic> {
    if node.top_level() && result.top_level_no_op_result {
        Some(lsp_types::Diagnostic {
            range: state
                .top_level_no_op
                .clone()
                .expect("it was just determined that top_level_no_op was Some")
                .range,
            severity: Some(lsp_types::DiagnosticSeverity::HINT),
            message: "This node has no operator, but contains something that does have an operator"
                .to_owned(),
            ..Default::default()
        })
    } else {
        None
    }
}

fn or_in_child_node(
    node: &ksp_cfg_formatter::parser::Node<'_>,
    _state: &LinterState,
    _result: &mut LinterStateResult,
) -> Option<lsp_types::Diagnostic> {
    if node.name.clone().map_or(false, |name| name.len() > 1) && !node.top_level() {
        Some(lsp_types::Diagnostic {
            range: range_to_range(node.name.as_ref().expect("It was just determined that it is Some").get_pos()),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            message: "names separated by '|' is only interpreted as OR in a top level node. Here, it's interpreted literally.".to_owned(),
            ..Default::default()
        })
    } else {
        None
    }
}

fn op_in_noop(
    node: &ksp_cfg_formatter::parser::Node,
    state: &LinterState,
    result: &mut LinterStateResult,
) -> Option<lsp_types::Diagnostic> {
    if state.top_level_no_op.is_some() && node.operator.is_some() {
        result.top_level_no_op_result = true;
        Some(lsp_types::Diagnostic {
            range: range_to_range(
                node.operator
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
        })
    } else {
        None
    }
}
