use ksp_cfg_formatter::parser::nom::Severity;
use lsp_types::DiagnosticSeverity;

use crate::linter::{Diagnostic, RelatedInformation};
pub fn sev_to_sev(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
        Severity::Hint => DiagnosticSeverity::HINT,
    }
}

// Notice the ´-1´s to get correct 0-indexed position in VSCode
pub(crate) fn range_to_range(parser_range: ksp_cfg_formatter::parser::Range) -> lsp_types::Range {
    lsp_types::Range::new(
        lsp_types::Position::new(parser_range.start.line - 1, parser_range.start.col - 1),
        lsp_types::Position::new(parser_range.end.line - 1, parser_range.end.col - 1),
    )
}

impl From<&Diagnostic> for lsp_types::Diagnostic {
    fn from(val: &Diagnostic) -> Self {
        Self {
            range: range_to_range(val.range),
            severity: val.severity.clone().map(crate::utils::sev_to_sev),
            source: val.source.clone(),
            message: val.message.clone(),
            related_information: val.related_information.as_ref().and_then(|v| {
                Some(
                    v.clone()
                        .into_iter()
                        .map(lsp_types::DiagnosticRelatedInformation::from)
                        .collect(),
                )
            }),
            ..Default::default()
        }
    }
}

impl From<RelatedInformation> for lsp_types::DiagnosticRelatedInformation {
    fn from(value: RelatedInformation) -> Self {
        Self {
            location: lsp_types::Location {
                uri: value.location.url,
                range: crate::utils::range_to_range(value.location.range),
            },
            message: value.message,
        }
    }
}
