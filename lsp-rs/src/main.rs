use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use lsp_types::{
    InitializeParams, OneOf, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

mod requests;
use requests::RequestDispatch;

mod notifications;
use notifications::NotificationDispatch;

mod linter;

use lsp_server::{Connection, Message, Request, Response};

fn main() -> anyhow::Result<()> {
    // Note that we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        // List of server capabilities
        document_formatting_provider: Some(OneOf::Left(true)),
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
    main_loop(&connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: &Connection, params: serde_json::Value) -> anyhow::Result<()> {
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

type HandlerFn = fn(&mut State, Response) -> anyhow::Result<()>;
struct State {
    data_base: DocumentDataBase,
    sender: Sender<lsp_server::Message>,
    outgoing: Outgoing,
    pending_requests: HashMap<lsp_server::RequestId, HandlerFn>,
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
        handler: HandlerFn,
    ) -> anyhow::Result<()> {
        let id: lsp_server::RequestId = self.outgoing.get_next_id().into();
        let request = Request::new(id.clone(), R::METHOD.to_string(), params);
        self.pending_requests.insert(id, handler);
        self.sender.send(request.into())?;
        Ok(())
    }
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

struct Outgoing {
    next_id: i32,
}

impl Outgoing {
    const fn new() -> Self {
        Self { next_id: 0 }
    }

    fn get_next_id(&mut self) -> i32 {
        let val = self.next_id;
        self.next_id += 1;
        val
    }
}