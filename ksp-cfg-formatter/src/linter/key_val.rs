use crate::parser::{AssignmentOperator, Range};

use super::{Diagnostic, Lintable, LinterState, LinterStateResult};

impl<'a> Lintable for crate::parser::KeyVal<'a> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        if let Some(diag) = op_in_noop_node(self, state, &mut result) {
            items.push(diag);
        }
        // The keyval has no operator, but uses MM logic in the key
        let mut diagnostics = noop_but_mm(self);
        if !diagnostics.is_empty() {
            items.append(&mut diagnostics);
        }

        (items, Some(result))
    }
}

fn op_in_noop_node(
    key_val: &crate::parser::KeyVal<'_>,
    state: &LinterState,
    result: &mut LinterStateResult,
) -> Option<Diagnostic> {
    if state.top_level_no_op.is_some() && key_val.operator.is_some() {
        result.top_level_no_op_result = true;
        Some(Diagnostic {
            range: key_val
                .operator
                .as_ref()
                .expect("it was just determined that the operator existed")
                .get_range(),

            severity: Some(crate::parser::Severity::Warning),
            source: Some("Unexpected_operator".to_owned()),
            message: "Key has operator, even though the top level does not!".to_owned(),
            related_information: Some(vec![super::RelatedInformation {
                location: state
                    .top_level_no_op
                    .clone()
                    .expect("It was just determined that the top_level_no_op is Some"),
                message: "This is where it happened".to_owned(),
            }]),
        })
    } else {
        None
    }
}

fn range_for_rest_of_name(key_val: &crate::parser::KeyVal) -> Vec<crate::parser::Range> {
    let mut ranges = vec![];
    if let Some(ranged) = key_val.array_index.as_ref() {
        ranges.push(ranged.get_range());
    }
    // if let Some(ranged) = key_val.needs.as_ref() {
    //     ranges.push(ranged.get_range());
    // }
    if let Some(ranged) = key_val.index.as_ref() {
        ranges.push(ranged.get_range());
    }
    match key_val.assignment_operator.as_ref() {
        AssignmentOperator::Assign => (),
        _ => {
            ranges.push(key_val.assignment_operator.get_range());
        }
    }

    Range::combine_ranges(ranges)
}

// TODO: Are some MM things allowed?
fn noop_but_mm(key_val: &crate::parser::KeyVal) -> Vec<Diagnostic> {
    if key_val.operator.is_some() || key_val.path.is_some() {
        return vec![];
    }
    let ranges = range_for_rest_of_name(key_val);
    let mut diagnostics = vec![];
    for range in ranges {
        diagnostics.push(Diagnostic {
            range,
            severity: Some(crate::parser::Severity::Warning),
            message: "No operator on KeyVal, but MM is used. this is likely not correct"
                .to_string(),
            // TODO: Add related info for start of KV
            ..Default::default()
        });
    }
    diagnostics
}
