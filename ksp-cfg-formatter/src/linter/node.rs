use crate::parser::{Node, Ranged};

use super::{Diagnostic, Lintable, LinterState, LinterStateResult, Location, RelatedInformation};

impl<'a> Lintable for crate::parser::Ranged<Node<'a>> {
    fn lint(&self, state: &LinterState) -> (Vec<Diagnostic>, Option<LinterStateResult>) {
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
        // The node has no operator, but uses MM logic in the identifier
        items.append(&mut noop_but_mm(self, state));

        let mut state: LinterState = state.clone();
        // Check for operators in nodes that do not have any operators
        if self.top_level() && self.operator.is_none() {
            state.top_level_no_op = Some(super::Location {
                url: state.this_url.clone(),
                range: self.get_range(),
            });
            // state.top_level_no_op = Some(self.get_range());
        }

        if let Some(name) = &self.name {
            if name.is_empty() {
                items.push(Diagnostic {
                    range: name.get_range(),
                    severity: Some(crate::parser::Severity::Info),
                    message: "Expected Name".to_owned(),
                    ..Default::default()
                });
            }
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
    node: &Node<'_>,
    state: &LinterState,
    result: &LinterStateResult,
) -> Option<Diagnostic> {
    if node.top_level() && result.top_level_no_op_result {
        Some(Diagnostic {
            range: state
                .top_level_no_op
                .clone()
                .expect("it was just determined that top_level_no_op was Some")
                .range,
            severity: Some(crate::parser::Severity::Hint),
            message: "This node has no operator, but contains something that does have an operator"
                .to_owned(),
            ..Default::default()
        })
    } else {
        None
    }
}

fn or_in_child_node(
    node: &Node<'_>,
    _state: &LinterState,
    _result: &mut LinterStateResult,
) -> Option<Diagnostic> {
    if let Some(name) = &node.name && name.len() > 1 && !node.top_level() {
        Some(Diagnostic {
            range: name.get_range(),
            severity: Some(crate::parser::Severity::Warning),
            message: "names separated by '|' is only interpreted as OR in a top level node. Here, it's interpreted literally.".to_owned(),
            ..Default::default()
        })
    } else {
        None
    }
}

fn op_in_noop(
    node: &Node,
    state: &LinterState,
    result: &mut LinterStateResult,
) -> Option<Diagnostic> {
    if let Some(top_level_no_op) = &state.top_level_no_op
    && let Some(operator) = &node.operator {
        result.top_level_no_op_result = true;
        Some(Diagnostic {
            range: operator.get_range(),
            severity: Some(crate::parser::Severity::Warning),
            message: "Node has operator, but top level does not!".to_owned(),
            related_information: Some(vec![super::RelatedInformation {
                location: top_level_no_op
                    .clone(),
                message: "This is where it happened".to_owned(),
            }]),
            source: Some("Unexpected_operator".to_owned()),
        })
    } else {
        None
    }
}

// TODO: Are there some MM things that are allowed?
fn range_for_rest_of_id(node: &Node) -> Vec<crate::parser::Range> {
    let mut ranges = vec![];
    if let Some(ranged) = node.name.as_ref() {
        ranges.push(ranged.get_range());
    }
    if let Some(ranged) = node.has.as_ref() {
        ranges.push(ranged.get_range());
    }
    // if let Some(ranged) = node.needs.as_ref() {
    //     ranges.push(ranged.get_range());
    // }
    if let Some(ranged) = node.index.as_ref() {
        ranges.push(ranged.get_range());
    }
    if let Some(ranged) = node.pass.as_ref() {
        ranges.push(ranged.get_range());
    }

    crate::parser::Range::combine_ranges(ranges)
}

fn noop_but_mm(node: &Ranged<Node>, state: &LinterState) -> Vec<Diagnostic> {
    if node.operator.is_some() || node.path.is_some() {
        return vec![];
    }
    let ranges = range_for_rest_of_id(node);
    let mut diagnostics = vec![];
    for range in ranges {
        diagnostics.push(Diagnostic {
            range,
            severity: Some(crate::parser::Severity::Warning),
            message:
                "No operator on Node, but MM is used in the identifier. this is likely not correct"
                    .to_string(),
            related_information: Some(vec![RelatedInformation {
                location: Location {
                    range: node.get_range(),
                    url: state.this_url.clone(),
                },
                message: "Expected operator here".to_owned(),
            }]),
            ..Default::default()
        });
    }
    if !diagnostics.is_empty() {
        diagnostics.push(Diagnostic {
            range: node.get_range().to_start(),
            severity: Some(crate::parser::Severity::Hint),
            message: "This node identifier contains MM, but has no operator".to_owned(),
            ..Default::default()
        });
    }
    diagnostics
}
