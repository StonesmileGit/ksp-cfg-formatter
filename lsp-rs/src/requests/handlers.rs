use log::debug;
use lsp_types::{DiagnosticRelatedInformation, Location};

use super::State;

pub(crate) fn handle_formatting_request(
    state: &mut State,
    params: lsp_types::DocumentFormattingParams,
) -> anyhow::Result<Option<Vec<lsp_types::TextEdit>>> {
    // let (id, params) = cast_request::<Formatting>(req)?;
    let url = params.text_document.uri;
    let path = url
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file"))?;

    // state.send_request::<lsp_types::request::WorkspaceConfiguration>(
    //     lsp_types::ConfigurationParams {
    //         items: vec![lsp_types::ConfigurationItem {
    //             // scope_uri: Some(url),
    //             scope_uri: None,
    //             section: Some("[ksp-cfg]".to_string()),
    //         }],
    //     },
    //     |state, response| {
    //         // debug!("got editor cfg with context response:\n{response:?}");
    //         debug!("Settings are now: {:?}", state.settings);
    //         Ok(())
    //     },
    // )?;
    let tabs = !params.options.insert_spaces;
    let tab_size = params.options.tab_size;
    let text = state
        .data_base
        .data_base
        .get(&path)
        .ok_or_else(|| anyhow::format_err!("Document not found"))?;

    // This is where the formatting should be done, by passing in settings and the ´text´
    debug!("formatting text:\n{text}\nwith settings tabs: `{tabs}`, tab size: `{tab_size}`\nother settings: {:?}\n", params.options.properties);
    let indentation = if tabs {
        ksp_cfg_formatter::Indentation::Tabs
    } else {
        ksp_cfg_formatter::Indentation::Spaces(tab_size as usize)
    };
    let new_text = ksp_cfg_formatter::Formatter::new(
        indentation,
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
    let uri = params.text_document.uri;
    let key = uri
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
        use lsp_types::DiagnosticSeverity as lsp_sev;
        disp_errors.push(lsp_types::Diagnostic {
            range: crate::utils::range_to_range(error.range),
            severity: Some(crate::utils::sev_to_sev(&error.severity)),
            message: error.message,
            related_information: error.context.clone().map(|context| {
                vec![DiagnosticRelatedInformation {
                    location: Location {
                        range: crate::utils::range_to_range(context.get_range()),
                        uri: uri.clone(),
                    },
                    message: context.to_string(),
                }]
            }),
            ..Default::default()
        });
        if let Some(context) = error.context {
            disp_errors.push(lsp_types::Diagnostic {
                range: crate::utils::range_to_range(context.get_range()),
                severity: Some(lsp_sev::HINT),
                message: context.to_string(),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        range: crate::utils::range_to_range(error.range),
                        uri: uri.clone(),
                    },
                    message: "original diagnostic".to_string(),
                }]),
                ..Default::default()
            });
        }
    }
    let mut items = ksp_cfg_formatter::linter::lint_ast(&doc, Some(uri))
        .iter()
        .map(crate::utils::diag_to_diag)
        .collect();
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
