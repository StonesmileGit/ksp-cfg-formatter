use super::State;

pub(crate) fn add_document_to_db(
    state: &mut State,
    not: lsp_types::DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.add_document_to_db(not)
}

pub(crate) fn update_document_in_db(
    state: &mut State,
    not: lsp_types::DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.update_document_in_db(not)
}

pub(crate) fn remove_document_from_db(
    state: &mut State,
    not: lsp_types::DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.remove_document_from_db(not)
}

pub(crate) fn handle_did_change_configuration(
    state: &mut State,
    _params: lsp_types::DidChangeConfigurationParams,
) -> anyhow::Result<()> {
    request_workspace_config(state)?;
    Ok(())
}

fn request_workspace_config(state: &mut State) -> anyhow::Result<()> {
    let a: lsp_types::ConfigurationParams = lsp_types::ConfigurationParams {
        items: vec![lsp_types::ConfigurationItem {
            scope_uri: None,
            section: Some("KspCfgLspServer".to_string()),
        }],
    };
    state.send_request::<lsp_types::request::WorkspaceConfiguration>(a, |_state, response| {
        eprintln!("got response to config request:\n{response:?}");
        Ok(())
    })?;
    Ok(())
}
