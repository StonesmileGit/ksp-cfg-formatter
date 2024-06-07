use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use log::{debug, info, warn};
// use log4rs::{
//     append::console::{ConsoleAppender, Target},
//     config::{Appender, Root},
//     encode::pattern::PatternEncoder,
//     Config,
// };
use lsp_types::{
    InitializeParams, OneOf, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

mod requests;
use requests::RequestDispatch;

mod notifications;
use notifications::NotificationDispatch;

mod utils;

use lsp_server::{Connection, Message, Request, Response};

fn main() -> anyhow::Result<()> {
    // let stderr = ConsoleAppender::builder()
    //     .target(Target::Stderr)
    //     .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
    //     .build();
    // let config = Config::builder()
    //     .appender(Appender::builder().build("stderr", Box::new(stderr)))
    //     .build(
    //         Root::builder()
    //             .appender("stderr")
    //             .build(log::LevelFilter::Trace), // Log4rs doesn't filter anything
    //     )
    //     .unwrap();
    // log4rs::init_config(config).unwrap();
    info!("Starting KSP Language Server\n");
    eprintln!("Starting WASM server");
    let (connection, io_threads) = Connection::stdio();
    eprintln!("stdio established");

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
    eprintln!("Starting main loop...");
    main_loop(&connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    info!("shutting down server\n");
    Ok(())
}

fn main_loop(connection: &Connection, params: serde_json::Value) -> anyhow::Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut state = State::new(connection.sender.clone());
    // Start by getting settings from the client
    notifications::handlers::handle_did_change_configuration(
        &mut state,
        &lsp_types::DidChangeConfigurationParams {
            settings: serde_json::Value::Null,
        },
    )?;
    eprintln!("Going into loop:");
    for msg in &connection.receiver {
        eprintln!("Handling message");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                RequestDispatch::new(&mut state, Some(req)).run()?;
            }
            Message::Response(resp) => {
                if let Some(handler) = state.pending_requests.remove(&resp.id) {
                    handler(&mut state, resp)?;
                } else {
                    warn!("got a response that was not in the queue!\n{:?}\n", resp);
                }
            }
            Message::Notification(not) => {
                NotificationDispatch::new(&mut state, Some(not)).run()?;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct Settings {
    use_tabs: bool,
    indent_size: u64,
    should_collapse: Option<bool>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            use_tabs: Default::default(),
            indent_size: Default::default(),
            should_collapse: Some(true),
        }
    }
}

type HandlerFn = fn(&mut State, Response) -> anyhow::Result<()>;
struct State {
    data_base: DocumentDataBase,
    settings: Settings,
    sender: Sender<lsp_server::Message>,
    outgoing: Outgoing,
    pending_requests: HashMap<lsp_server::RequestId, HandlerFn>,
}

impl State {
    fn new(sender: Sender<lsp_server::Message>) -> Self {
        Self {
            data_base: DocumentDataBase::default(),
            settings: Settings::default(),
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
        params: &lsp_types::DidOpenTextDocumentParams,
    ) -> anyhow::Result<()> {
        let key = params
            .text_document
            .uri
            .to_file_path()
            .map_err(|()| anyhow::format_err!("url is not a file"))?;
        self.data_base
            .insert(key.clone(), params.text_document.text.clone());
        debug!("inserted file {key:?}");
        Ok(())
    }
    fn update_document_in_db(
        &mut self,
        params: &lsp_types::DidChangeTextDocumentParams,
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
        debug!("{:?}", self.data_base.get(&key));
        Ok(())
    }
    fn remove_document_from_db(
        &mut self,
        params: &lsp_types::DidCloseTextDocumentParams,
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
