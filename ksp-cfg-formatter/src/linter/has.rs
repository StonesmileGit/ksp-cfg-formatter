use itertools::Itertools;

use crate::parser::Ranged;

use super::{Diagnostic, Lintable};

impl<'a> Lintable for crate::parser::HasBlock<'a> {
    fn lint(
        &self,
        state: &super::LinterState,
    ) -> (Vec<Diagnostic>, Option<super::LinterStateResult>) {
        (
            self.predicates
                .iter()
                .flat_map(|pred| pred.lint(state).0)
                .collect_vec(),
            None,
        )
    }
}

impl<'a> Lintable for Ranged<crate::parser::HasPredicate<'a>> {
    fn lint(
        &self,
        state: &super::LinterState,
    ) -> (Vec<Diagnostic>, Option<super::LinterStateResult>) {
        use crate::parser::HasPredicate as HP;
        let mut items = vec![];
        match self.as_ref() {
            HP::NodePredicate {
                negated: _,
                node_type: _,
                name: _,
                has_block,
            } => {
                if let Some(has_block) = has_block {
                    let (mut diagnostics, _res) = has_block.lint(state);
                    items.append(&mut diagnostics);
                }
            }
            HP::KeyPredicate {
                negated: _,
                key: _,
                value,
                match_type: _,
            } => {
                if let Some(value) = value {
                    if value.is_empty() {
                        items.push(Diagnostic {
                            range: value.get_range(),
                            severity: Some(crate::parser::Severity::Info),
                            message: "Expected value".to_owned(),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        (items, None)
    }
}
