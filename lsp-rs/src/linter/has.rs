use ksp_cfg_formatter::parser::Ranged;
use lsp_types::Diagnostic;

use crate::linter::range_to_range;

use super::Lintable;

impl<'a> Lintable for ksp_cfg_formatter::parser::HasBlock<'a> {
    fn lint(
        &self,
        state: &super::LinterState,
    ) -> (Vec<lsp_types::Diagnostic>, Option<super::LinterStateResult>) {
        let mut items = vec![];
        for pred in &self.predicates {
            let (mut diagnostics, _res) = pred.lint(state);
            items.append(&mut diagnostics);
        }
        (items, None)
    }
}

impl<'a> Lintable for Ranged<ksp_cfg_formatter::parser::HasPredicate<'a>> {
    fn lint(
        &self,
        state: &super::LinterState,
    ) -> (Vec<lsp_types::Diagnostic>, Option<super::LinterStateResult>) {
        use ksp_cfg_formatter::parser::HasPredicate as HP;
        let mut items = vec![];
        match self.as_ref() {
            HP::NodePredicate {
                negated,
                node_type,
                name,
                has_block,
            } => {
                if let Some(has_block) = has_block {
                    let (mut diagnostics, _res) = has_block.lint(state);
                    items.append(&mut diagnostics);
                }
            }
            HP::KeyPredicate {
                negated,
                key,
                value,
                match_type,
            } => {
                if let Some(value) = value {
                    if value.is_empty() {
                        items.push(Diagnostic {
                            range: range_to_range(self.get_range()),
                            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
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
