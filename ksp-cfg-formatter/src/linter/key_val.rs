use crate::parser::{AssignmentOperator, KeyVal, Operator, Range, Ranged};

use super::{Diagnostic, Lintable, LinterState, LinterStateResult, Location, RelatedInformation};

impl<'a> Lintable for Ranged<KeyVal<'a>> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
        let mut items = vec![];
        let mut result = LinterStateResult {
            top_level_no_op_result: false,
        };
        if let Some(diag) = op_in_noop_node(self, state, &mut result) {
            items.push(diag);
        }
        // The keyval has no operator, but uses MM logic in the key
        items.append(&mut noop_but_mm(self, state));
        // Regex was used without the operator being Edit
        items.append(&mut check_regex_not_edit(self, state));

        (items, Some(result))
    }
}

fn op_in_noop_node(
    key_val: &KeyVal<'_>,
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

fn check_regex_not_edit(key_val: &KeyVal, _state: &LinterState) -> Vec<Diagnostic> {
    if matches!(
        key_val.assignment_operator.as_ref(),
        AssignmentOperator::RegexReplace
    ) {
        if let Some(op) = key_val.operator.clone() {
            if matches!(op.as_ref(), Operator::Edit) {
                return vec![];
            }
        }
        // This is where the error is returned
        return vec![Diagnostic {
            message: "Regex replace  assignment operation was used without the key-val operator being Edit".to_string(),
            range: key_val.assignment_operator.get_range(),
            // TODO: Add info about the start of the KV
            related_information: None,
            severity: Some(crate::parser::Severity::Warning),
            ..Default::default()
        }];
    }
    vec![]
}

// TODO: Are some MM things allowed?
fn range_for_rest_of_name(key_val: &KeyVal) -> Vec<crate::parser::Range> {
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

fn noop_but_mm(key_val: &Ranged<KeyVal>, state: &LinterState) -> Vec<Diagnostic> {
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
            related_information: Some(vec![RelatedInformation {
                location: Location {
                    range: key_val.get_range(),
                    url: state.this_url.clone(),
                },
                message: "Expected operator here".to_owned(),
            }]),
            ..Default::default()
        });
    }
    if !diagnostics.is_empty() {
        diagnostics.push(Diagnostic {
            range: key_val.get_range().to_start(),
            severity: Some(crate::parser::Severity::Hint),
            message: "This key contains MM, but has no operator".to_owned(),
            ..Default::default()
        });
    }
    diagnostics
}
