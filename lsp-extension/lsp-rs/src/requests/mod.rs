use super::State;
use log::error;
use lsp_server::{ExtractError, Message, Response};

mod handlers;

pub(crate) struct RequestDispatch<'a> {
    state: &'a mut State,
    request: Option<lsp_server::Request>,
}

impl<'a> RequestDispatch<'a> {
    pub(crate) fn new(state: &'a mut State, request: Option<lsp_server::Request>) -> Self {
        Self { state, request }
    }
    pub(crate) fn run(self) -> anyhow::Result<()> {
        use lsp_types::request as reqs;
        self.handle_request::<reqs::Formatting>(handlers::handle_formatting_request)?
            .handle_request::<reqs::DocumentDiagnosticRequest>(
                handlers::handle_diagnostics_request,
            )?
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
        let Some(req) = self.request.take()else {
            return Ok(self)
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
                error!("unhandled request: {req:?}\n");
            }
        }
    }
}
