use crate::parser::Ranged;

use super::{Diagnostic, Lintable};

impl<'a> Lintable for crate::parser::HasBlock<'a> {
    fn lint(
        &self,
        state: &super::LinterState,
    ) -> (Vec<Diagnostic>, Option<super::LinterStateResult>) {
        let mut items = vec![];
        for pred in &self.predicates {
            let (mut diagnostics, _res) = pred.lint(state);
            items.append(&mut diagnostics);
        }
        (items, None)
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
                            severity: Some(crate::parser::nom::Severity::Warning),
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
