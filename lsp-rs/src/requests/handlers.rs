use super::State;

pub(crate) fn handle_formatting_request(
    state: &mut State,
    params: lsp_types::DocumentFormattingParams,
) -> anyhow::Result<Option<Vec<lsp_types::TextEdit>>> {
    // let (id, params) = cast_request::<Formatting>(req)?;
    let path = params
        .text_document
        .uri
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file"))?;
    let tabs = !params.options.insert_spaces;
    let tab_size = params.options.tab_size;
    let text = state
        .data_base
        .data_base
        .get(&path)
        .ok_or_else(|| anyhow::format_err!("Document not found"))?;

    // This is where the formatting should be done, by passing in settings and the ´text´
    eprintln!("formatting text `{text}` with settings tabs: `{tabs}`, tab size: `{tab_size}`");
    eprintln!("other settings: {:?}", params.options.properties);
    let new_text = ksp_cfg_formatter::Formatter::new(
        ksp_cfg_formatter::Indentation::Tabs,
        false,
        ksp_cfg_formatter::LineReturn::Identify,
    )
    .fail_silent()
    .format_text(text);

    let text_edit = text_edit_entire_document(text, new_text)?;
    let edit = vec![text_edit];
    Ok(Some(edit))
}

/// Takes the orignal text and the new text and creates a single edit replacing the entire document
fn text_edit_entire_document(original: &str, new: String) -> anyhow::Result<lsp_types::TextEdit> {
    Ok(lsp_types::TextEdit {
        range: lsp_types::Range {
            start: lsp_types::Position {
                line: 0,
                character: 0,
            },
            end: lsp_types::Position {
                // TODO: Is u32::MAX good enough?
                line: original.lines().count().try_into()?,
                character: original
                    .lines()
                    .last()
                    .unwrap_or_default()
                    .chars()
                    .count()
                    .try_into()?,
            },
        },
        new_text: new,
    })
}

pub(crate) fn handle_diagnostics_request(
    state: &mut State,
    params: lsp_types::DocumentDiagnosticParams,
) -> anyhow::Result<lsp_types::DocumentDiagnosticReportResult> {
    let key = params
        .text_document
        .uri
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file"))?;
    let text = state
        .data_base
        .data_base
        .get(&key)
        .ok_or_else(|| anyhow::format_err!("no text provided"))?;
    let (doc, errors) = ksp_cfg_formatter::parser::nom::parse(text);
    let mut disp_errors = vec![];
    for error in errors {
        disp_errors.push(lsp_types::Diagnostic {
            range: crate::linter::range_to_range(error.range),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            message: error.message,
            ..Default::default()
        })
    }
    let mut items = crate::linter::lint_ast(&doc, params.text_document.uri);
    disp_errors.append(&mut items);
    Ok(lsp_types::DocumentDiagnosticReportResult::Report(
        lsp_types::DocumentDiagnosticReport::Full(lsp_types::RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: lsp_types::FullDocumentDiagnosticReport {
                result_id: None,
                items: disp_errors,
            },
        }),
    ))
}
