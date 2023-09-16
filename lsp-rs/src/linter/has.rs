use super::Lintable;

impl<'a> Lintable for ksp_cfg_formatter::parser::HasBlock<'a> {
    fn lint(
        &self,
        _state: &super::LinterState,
    ) -> (Vec<lsp_types::Diagnostic>, Option<super::LinterStateResult>) {
        (vec![], None)
    }
}

impl<'a> Lintable for ksp_cfg_formatter::parser::HasPredicate<'a> {
    fn lint(
        &self,
        _state: &super::LinterState,
    ) -> (Vec<lsp_types::Diagnostic>, Option<super::LinterStateResult>) {
        (vec![], None)
    }
}
