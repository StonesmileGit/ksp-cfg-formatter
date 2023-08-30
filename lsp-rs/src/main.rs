use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use lsp_types::{
    CompletionOptions, InitializeParams, OneOf, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};

use lsp_server::{Connection, ExtractError, Message, Request, Response};

fn main() -> anyhow::Result<()> {
    // Note that we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        // List of server capabilities
        definition_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            ..Default::default()
        }),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        diagnostic_provider: Some(lsp_types::DiagnosticServerCapabilities::Options(
            lsp_types::DiagnosticOptions {
                identifier: Some("test_identifier_diagnostics".to_owned()),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
                work_done_progress_options: lsp_types::WorkDoneProgressOptions {
                    work_done_progress: None,
                },
            },
        )),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> anyhow::Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut state = State::new(connection.sender.clone());
    eprintln!("starting example main loop");
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                RequestDispatch::new(&mut state, Some(req)).run()?;
            }
            Message::Response(resp) => match state.pending_requests.remove(&resp.id) {
                Some(handler) => handler(&mut state, resp)?,
                None => eprintln!("got a response that was not in the queue!"),
            },
            Message::Notification(not) => {
                NotificationDispatch::new(&mut state, Some(not)).run()?;
            }
        }
    }
    Ok(())
}

struct State {
    data_base: DocumentDataBase,
    sender: Sender<lsp_server::Message>,
    outgoing: Outgoing,
    pending_requests:
        HashMap<lsp_server::RequestId, fn(&mut State, Response) -> anyhow::Result<()>>,
}

impl State {
    fn new(sender: Sender<lsp_server::Message>) -> Self {
        Self {
            data_base: DocumentDataBase::default(),
            sender,
            outgoing: Outgoing::new(),
            pending_requests: HashMap::default(),
        }
    }

    fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: fn(&mut State, Response) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let id: lsp_server::RequestId = self.outgoing.get_next_id().into();
        let request = Request::new(id.clone(), R::METHOD.to_string(), params);
        self.pending_requests.insert(id, handler);
        self.sender.send(request.into())?;
        Ok(())
    }
}

struct RequestDispatch<'a> {
    state: &'a mut State,
    request: Option<lsp_server::Request>,
}

impl<'a> RequestDispatch<'a> {
    fn new(state: &'a mut State, request: Option<lsp_server::Request>) -> Self {
        Self { state, request }
    }
    fn run(self) -> anyhow::Result<()> {
        use lsp_types::request as reqs;
        self.handle_request::<reqs::Formatting>(handle_formatting_request)?
            .handle_request::<reqs::DocumentDiagnosticRequest>(handle_diagnostics_request)?
            .finish();
        Ok(())
    }
    fn handle_request<R>(
        mut self,
        f: fn(&mut State, R::Params) -> anyhow::Result<R::Result>,
    ) -> anyhow::Result<Self>
    where
        R: lsp_types::request::Request,
        R::Params: serde::de::DeserializeOwned,
    {
        let req = match self.request.take() {
            Some(it) => it,
            None => return Ok(self),
        };
        let (id, params) = match req.extract::<R::Params>(R::METHOD) {
            Ok(it) => it,
            Err(ExtractError::JsonError { method, error }) => {
                panic!("Invalid request\nMethod: {method}\n error: {error}",)
            }
            Err(ExtractError::MethodMismatch(req)) => {
                self.request = Some(req);
                return Ok(self);
            }
        };
        let result = f(self.state, params)?;
        self.state
            .sender
            .send(Message::Response(Response::new_ok(id, result)))?;
        Ok(self)
    }

    fn finish(&mut self) {
        if let Some(req) = &self.request {
            if !req.method.starts_with("$/") {
                eprintln!("unhandled request: {:?}", req);
            }
        }
    }
}

struct NotificationDispatch<'a> {
    state: &'a mut State,
    notification: Option<lsp_server::Notification>,
}

impl<'a> NotificationDispatch<'a> {
    fn new(state: &'a mut State, notification: Option<lsp_server::Notification>) -> Self {
        Self {
            state,
            notification,
        }
    }
    fn run(self) -> anyhow::Result<()> {
        use lsp_types::notification as notif;
        self.handle_notification::<notif::DidOpenTextDocument>(add_document_to_db)?
            .handle_notification::<notif::DidChangeTextDocument>(update_document_in_db)?
            .handle_notification::<notif::DidCloseTextDocument>(remove_document_from_db)?
            .handle_notification::<notif::DidChangeConfiguration>(handle_did_change_configuration)?
            .finish();
        Ok(())
    }

    fn handle_notification<N>(
        mut self,
        f: fn(&mut State, N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self>
    where
        N: lsp_types::notification::Notification,
        N::Params: serde::de::DeserializeOwned,
    {
        let not = match self.notification.take() {
            Some(it) => it,
            None => return Ok(self),
        };
        let params = match not.extract::<N::Params>(N::METHOD) {
            Ok(it) => it,
            Err(ExtractError::JsonError { method, error }) => {
                panic!("Invalid notification\nMethod: {method}\n error: {error}",)
            }
            Err(ExtractError::MethodMismatch(not)) => {
                self.notification = Some(not);
                return Ok(self);
            }
        };
        f(&mut self.state, params)?;
        Ok(self)
    }

    fn finish(&mut self) {
        if let Some(not) = &self.notification {
            if !not.method.starts_with("$/") {
                eprintln!("unhandled notification: {:?}", not);
            }
        }
    }
}

fn add_document_to_db(
    state: &mut State,
    not: lsp_types::DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.add_document_to_db(not)
}

fn update_document_in_db(
    state: &mut State,
    not: lsp_types::DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.update_document_in_db(not)
}

fn remove_document_from_db(
    state: &mut State,
    not: lsp_types::DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    state.data_base.remove_document_from_db(not)
}

#[derive(Default)]
struct DocumentDataBase {
    data_base: HashMap<PathBuf, String>,
}

impl DocumentDataBase {
    fn add_document_to_db(
        &mut self,
        params: lsp_types::DidOpenTextDocumentParams,
    ) -> anyhow::Result<()> {
        let key = params
            .text_document
            .uri
            .to_file_path()
            .map_err(|()| anyhow::format_err!("url is not a file"))?;
        self.data_base
            .insert(key.clone(), params.text_document.text);
        eprintln!("inserted file {key:?}");
        Ok(())
    }
    fn update_document_in_db(
        &mut self,
        params: lsp_types::DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        let key = params
            .text_document
            .uri
            .to_file_path()
            .map_err(|()| anyhow::format_err!("url is not a file"))?;
        self.data_base.insert(
            key.clone(),
            params
                .content_changes
                .iter()
                .map(|a| a.text.as_str())
                .collect(),
        );
        eprintln!("{:?}", self.data_base.get(&key));
        Ok(())
    }
    fn remove_document_from_db(
        &mut self,
        params: lsp_types::DidCloseTextDocumentParams,
    ) -> anyhow::Result<()> {
        let key = params
            .text_document
            .uri
            .to_file_path()
            .map_err(|()| anyhow::format_err!("url is not a file"))?;
        self.data_base.remove(&key);
        Ok(())
    }
}

fn handle_formatting_request(
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
        .ok_or(anyhow::format_err!("Document not found"))?;

    // This is where the formatting should be done, by passing in settings and the ´text´
    eprintln!("formatting text `{text}` with settings tabs: `{tabs}`, tab size: `{tab_size}`");
    eprintln!("other settings: {:?}", params.options.properties);
    let new_text = "abc".to_owned();

    let text_edit = text_edit_entire_document(text, new_text)?;
    let edit = vec![text_edit];
    Ok(Some(edit))
}

fn handle_diagnostics_request(
    state: &mut State,
    params: lsp_types::DocumentDiagnosticParams,
) -> anyhow::Result<lsp_types::DocumentDiagnosticReportResult> {
    let key = params
        .text_document
        .uri
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file"))?;
    let text = state.data_base.data_base.get(&key);
    let mut items = vec![];
    items.push(lsp_types::Diagnostic {
        range: lsp_types::Range::default(),
        severity: Some(lsp_types::DiagnosticSeverity::INFORMATION),
        code: Some(lsp_types::NumberOrString::Number(1)),
        code_description: None,
        source: Some("source_text".to_owned()),
        message: "message".to_owned(),
        related_information: None,
        tags: None,
        data: None,
    });
    Ok(lsp_types::DocumentDiagnosticReportResult::Report(
        lsp_types::DocumentDiagnosticReport::Full(lsp_types::RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: lsp_types::FullDocumentDiagnosticReport {
                result_id: None,
                items,
            },
        }),
    ))
}

/// Takes the orignal text and the new text and creates a single edit replacing the entire document
fn text_edit_entire_document(
    original: &String,
    new: String,
) -> anyhow::Result<lsp_types::TextEdit> {
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

fn handle_did_change_configuration(
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

struct Outgoing {
    next_id: i32,
}

impl Outgoing {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    fn get_next_id(&mut self) -> i32 {
        let val = self.next_id;
        self.next_id += 1;
        val
    }
}
