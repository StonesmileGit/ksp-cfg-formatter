use log::{debug, error};

use super::State;

pub(crate) fn add_document_to_db(
    state: &mut State,
    not: &lsp_types::DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.add_document_to_db(not)
}

pub(crate) fn update_document_in_db(
    state: &mut State,
    not: &lsp_types::DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.update_document_in_db(not)
}

pub(crate) fn remove_document_from_db(
    state: &mut State,
    not: &lsp_types::DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.remove_document_from_db(not)
}

pub(crate) fn handle_did_change_configuration(
    state: &mut State,
    _params: &lsp_types::DidChangeConfigurationParams,
) -> anyhow::Result<()> {
    request_workspace_config(state)?;
    Ok(())
}

fn request_workspace_config(state: &mut State) -> anyhow::Result<()> {
    // Fetch editor configs
    state.send_request::<lsp_types::request::WorkspaceConfiguration>(
        lsp_types::ConfigurationParams {
            items: vec![lsp_types::ConfigurationItem {
                scope_uri: None,
                section: Some("editor".to_string()),
            }],
        },
        |_state, _response| {
            // response.result.map(|result| {
            //     if let serde_json::Value::Array(arr) = result {
            //         arr.first().map(|first| {
            //             if let serde_json::Value::Object(obj) = first {
            //                 obj.get("tabSize").map(|tabsize| {
            //                     if let serde_json::Value::Number(number) = tabsize {
            //                         number.as_u64().map(|n| state.settings.indent_size = n);
            //                     }
            //                 });
            //                 obj.get("insertSpaces").map(|use_spaces| {
            //                     if let serde_json::Value::Bool(use_spaces) = use_spaces {
            //                         state.settings.use_tabs = !use_spaces
            //                     }
            //                 });
            //             }
            //         });
            //     }
            // });
            // debug!("Settings are now: {:?}", state.settings);
            Ok(())
        },
    )?;
    // Fetch extension settings
    state.send_request::<lsp_types::request::WorkspaceConfiguration>(
        lsp_types::ConfigurationParams {
            items: vec![lsp_types::ConfigurationItem {
                scope_uri: None,
                section: Some("KspCfgLspServer".to_string()),
            }],
        },
        |state, response| {
            debug!("got KspCfgLspServer response:\n{response:?}\n");
            if let Some(serde_json::Value::Array(arr)) = response.result {
                if let Some(serde_json::Value::Object(obj)) = arr.first() {
                    if let Some(serde_json::Value::String(log_str)) = obj.get("logLevel") {
                        let log_level = match log_str.as_str() {
                            "off" => log::LevelFilter::Off,
                            "error" => log::LevelFilter::Error,
                            "warning" => log::LevelFilter::Warn,
                            "info" => log::LevelFilter::Info,
                            "debug" => log::LevelFilter::Debug,
                            "trace" => log::LevelFilter::Trace,
                            _ => {
                                error!("Parsing the logLevel setting failed! Defaulting to Info\n");
                                log::LevelFilter::Info
                            }
                        };
                        // TODO: Is it needed in the settings?
                        state.settings.log_level = log_level;
                        log::set_max_level(log_level);
                    }
                    if let Some(serde_json::Value::String(should_collapse)) = obj.get("shouldCollapse") {
                        let should_collapse = match should_collapse.as_str() {
                            "keep" => None,
                            "collapse" => Some(true),
                            "expand" => Some(false),
                            _ => {
                                error!("Parsing the shouldCollapse setting failed! Defaulting to should collapse\n");
                                Some(true)
                            }
                        };
                        state.settings.should_collapse = should_collapse;
                    }
                };
            };
            // let a = response.result.and_then(|res| {
            //     if let serde_json::Value::Array(arr) = res {
            //         Some(arr)
            //     } else {
            //         None
            //     }
            // });
            debug!("Settings are now: {:?}\n", state.settings);
            Ok(())
        },
    )?;
    Ok(())
}
