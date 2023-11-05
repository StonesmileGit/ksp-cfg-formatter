use ksp_cfg_formatter::parser::AssignmentOperator;

use super::{range_to_range, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for ksp_cfg_formatter::parser::KeyVal<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<lsp_types::Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        if let Some(diag) = op_in_noop_node(self, state, &mut result) {
            items.push(diag);
        }
        // The keyval has no operator, but uses MM logic in the key
        if let Some(diag) = noop_but_mm(self) {
            items.push(diag);
        }

        (items, Some(result))
    }
}

fn op_in_noop_node(
    key_val: &ksp_cfg_formatter::parser::KeyVal<'_>,
    state: &LinterState,
    result: &mut LinterStateResult,
) -> Option<lsp_types::Diagnostic> {
    if state.top_level_no_op.is_some() && key_val.operator.is_some() {
        result.top_level_no_op_result = true;
        Some(lsp_types::Diagnostic {
            range: range_to_range(
                key_val
                    .operator
                    .as_ref()
                    .expect("it was just determined that the operator existed")
                    .get_pos(),
            ),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            code: Some(lsp_types::NumberOrString::Number(1)),
            code_description: None,
            source: Some("Unexpected_operator".to_owned()),
            message: "Key has operator, even though the top level does not!".to_owned(),
            related_information: Some(vec![lsp_types::DiagnosticRelatedInformation {
                location: state
                    .top_level_no_op
                    .clone()
                    .expect("It was just determined that the top_level_no_op is Some"),
                message: "This is where it happened".to_owned(),
            }]),
            tags: None,
            data: None,
        })
    } else {
        None
    }
}

fn range_for_rest_of_name(
    key_val: &ksp_cfg_formatter::parser::KeyVal,
) -> Option<ksp_cfg_formatter::parser::Range> {
    let mut ranges = vec![];
    if let Some(ranged) = key_val.array_index.as_ref() {
        ranges.push(ranged.get_pos());
    }
    if let Some(ranged) = key_val.needs.as_ref() {
        ranges.push(ranged.get_pos());
    }
    if let Some(ranged) = key_val.index.as_ref() {
        ranges.push(ranged.get_pos());
    }
    match key_val.assignment_operator.as_ref() {
        AssignmentOperator::Assign => (),
        _ => {
            ranges.push(key_val.assignment_operator.get_pos());
        }
    }
    ranges.into_iter().reduce(|a, b| a + b)
}

// TODO: Are some MM things allowed?
fn noop_but_mm(key_val: &ksp_cfg_formatter::parser::KeyVal) -> Option<lsp_types::Diagnostic> {
    if key_val.operator.is_some() {
        return None;
    }
    if let Some(range) = range_for_rest_of_name(key_val) {
        return Some(lsp_types::Diagnostic {
            range: range_to_range(range),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            message: "No operator, but MM is used. this is likely not correct".to_string(),
            // TODO: Add related info for start of KV
            ..Default::default()
        });
    }
    None
}
