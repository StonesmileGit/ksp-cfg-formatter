use log::error;
use lsp_server::ExtractError;

use super::State;
pub(super) mod handlers;

pub(crate) struct NotificationDispatch<'a> {
    state: &'a mut State,
    notification: Option<lsp_server::Notification>,
}

impl<'a> NotificationDispatch<'a> {
    pub(crate) fn new(
        state: &'a mut State,
        notification: Option<lsp_server::Notification>,
    ) -> Self {
        Self {
            state,
            notification,
        }
    }
    pub(crate) fn run(self) -> anyhow::Result<()> {
        use lsp_types::notification as notif;
        self.handle_notification::<notif::DidOpenTextDocument>(handlers::add_document_to_db)?
            .handle_notification::<notif::DidChangeTextDocument>(handlers::update_document_in_db)?
            .handle_notification::<notif::DidCloseTextDocument>(handlers::remove_document_from_db)?
            .handle_notification::<notif::DidChangeConfiguration>(
                handlers::handle_did_change_configuration,
            )?
            .finish();
        Ok(())
    }

    fn handle_notification<N>(
        mut self,
        f: fn(&mut State, &N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self>
    where
        N: lsp_types::notification::Notification,
        N::Params: serde::de::DeserializeOwned,
    {
        let Some(not) = self.notification.take() else {
            return Ok(self)
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
        f(self.state, &params)?;
        Ok(self)
    }

    fn finish(&mut self) {
        if let Some(not) = &self.notification {
            if !not.method.starts_with("$/") {
                error!("unhandled notification: {not:?}\n");
            }
        }
    }
}
