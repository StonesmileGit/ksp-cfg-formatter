use ksp_cfg_formatter::parser::Severity;
use lsp_types::DiagnosticSeverity;

use ksp_cfg_formatter::linter::{Diagnostic, RelatedInformation};
pub const fn sev_to_sev(severity: &Severity) -> DiagnosticSeverity {
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

pub fn diag_to_diag(val: &Diagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: range_to_range(val.range),
        severity: val
            .severity
            .clone()
            .map(|sev| crate::utils::sev_to_sev(&sev)),
        source: val.source.clone(),
        message: val.message.clone(),
        related_information: val.related_information.as_ref().map(|v| {
            v.clone()
                .into_iter()
                .map(relinfo_to_relinfo)
                .filter_map(std::result::Result::ok)
                .collect()
        }),
        ..Default::default()
    }
}

pub fn relinfo_to_relinfo(
    value: RelatedInformation,
) -> Result<lsp_types::DiagnosticRelatedInformation, ()> {
    Ok(lsp_types::DiagnosticRelatedInformation {
        location: lsp_types::Location {
            uri: value.location.url.ok_or(())?,
            range: crate::utils::range_to_range(value.location.range),
        },
        message: value.message,
    })
}
