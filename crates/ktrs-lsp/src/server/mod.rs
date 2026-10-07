//! The server: initialization, the message loop and its request handlers (protocol surface: the crate docs).

mod notifications;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::Sender;
use ktrs_lint::LintError;
use ktrs_project::ProjectConfig;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::notification::{self as n, Notification as _};
use lsp_types::request::{self as r, Request as _};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionParams, CodeActionProviderCapability, DocumentFormattingParams, InitializeParams,
    InitializeResult, LogMessageParams, MessageType, OneOf, PublishDiagnosticsParams, SaveOptions, ServerCapabilities, ServerInfo,
    ShowMessageParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit,
    Uri,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::code_actions::{FIX_ALL_KIND, Linted, code_actions};
use crate::config::{Effective, Formatter, resolve};
use crate::diagnostics::diagnostics;
use crate::documents::Documents;
use crate::formatting::{ktfmt, ktfmt_options};
use crate::ktlint::{Failure, KtlintEngines, code, fix_all, lint};
use crate::settings::Settings;
use crate::text::text_edits;

/// Serves `connection` from `initialize` to `exit`.
pub fn serve(connection: Connection) -> Result<(), String> {
    let (id, params) = connection.initialize_start().map_err(|e| e.to_string())?;
    let params: InitializeParams = serde_json::from_value(params).map_err(|e| format!("invalid initialize params: {e}"))?;
    let result = InitializeResult {
        capabilities: capabilities(),
        server_info: Some(ServerInfo { name: "ktrs".to_owned(), version: Some(env!("CARGO_PKG_VERSION").to_owned()) }),
    };
    connection.initialize_finish(id, serde_json::to_value(result).expect("serializable")).map_err(|e| e.to_string())?;
    ktrs_lint::engine::silence_caught_rule_panics();
    let mut server = Server::new(&connection);
    if let Some(options) = &params.initialization_options {
        server.configure(options);
    }
    let watch = params.capabilities.workspace.and_then(|w| w.did_change_watched_files).and_then(|w| w.dynamic_registration);
    if watch == Some(true) {
        server.register_watchers();
    }
    server.run()
}

fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions { include_text: Some(false) })),
            ..TextDocumentSyncOptions::default()
        })),
        document_formatting_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX, CodeActionKind::new(FIX_ALL_KIND)]),
            ..CodeActionOptions::default()
        })),
        ..ServerCapabilities::default()
    }
}

struct Server<'c> {
    connection: &'c Connection,
    documents: Documents,
    settings: Settings,
    engines: KtlintEngines,
    /// The last configuration logged per build root.
    logged: HashMap<PathBuf, String>,
}

impl<'c> Server<'c> {
    fn new(connection: &'c Connection) -> Server<'c> {
        let sender = connection.sender.clone();
        let engine_warnings = Arc::new(move |logger: &str, message: &str| send_log(&sender, MessageType::WARNING, format!("{logger}: {message}")));
        Server { connection, documents: Documents::default(), settings: Settings::default(), engines: KtlintEngines::new(engine_warnings), logged: HashMap::new() }
    }

    /// Handles messages until `exit`. After each burst of queued messages, lints the documents that changed:
    /// a document edited several times in the burst is linted once, at its latest text.
    fn run(&mut self) -> Result<(), String> {
        while let Ok(message) = self.connection.receiver.recv() {
            if self.handle(message)? {
                return Ok(());
            }
            while let Ok(message) = self.connection.receiver.try_recv() {
                if self.handle(message)? {
                    return Ok(());
                }
            }
            for uri in self.documents.take_stale() {
                self.publish_diagnostics(&uri);
            }
        }
        Ok(())
    }

    /// Whether the server is done.
    fn handle(&mut self, message: Message) -> Result<bool, String> {
        match message {
            Message::Request(request) => {
                if self.connection.handle_shutdown(&request).map_err(|e| e.to_string())? {
                    return Ok(true);
                }
                let response = self.respond(request);
                self.send(response.into());
            }
            Message::Notification(notification) if notification.method == n::Exit::METHOD => return Ok(true),
            Message::Notification(notification) => self.notify(notification),
            Message::Response(_) => {}
        }
        Ok(false)
    }

    fn respond(&mut self, request: Request) -> Response {
        let id = request.id.clone();
        let result = match request.method.as_str() {
            r::Formatting::METHOD => params(request.params).map(|p| self.formatting(p)).and_then(to_value),
            r::CodeActionRequest::METHOD => params(request.params).map(|p| self.code_actions(p)).and_then(to_value),
            method => return Response::new_err(id, ErrorCode::MethodNotFound as i32, format!("unsupported request {method}")),
        };
        match result {
            Ok(value) => Response::new_ok(id, value),
            Err(message) => Response::new_err(id, ErrorCode::InvalidParams as i32, message),
        }
    }

    /// The configuration for `path`, logged when it is new or changed for its build root.
    fn effective(&mut self, path: Option<&Path>) -> Effective {
        let detected = match path {
            Some(path) => ktrs_project::detect(path),
            None => ProjectConfig { root: PathBuf::new(), format: None, ktlint: None, notes: vec!["not a file: no build files".to_owned()] },
        };
        let effective = resolve(detected, &self.settings);
        let description = effective.describe();
        if self.logged.get(&effective.root) != Some(&description) {
            self.log(MessageType::INFO, description.clone());
            self.logged.insert(effective.root.clone(), description);
        }
        effective
    }

    /// The document's lint errors (linted now unless cached); `None` without diagnostics or when linting failed.
    fn lint_errors(&mut self, uri: &Uri, effective: &Effective) -> Option<Vec<LintError>> {
        let config = effective.ktlint.as_ref()?;
        let document = self.documents.get(uri)?;
        if let Some(errors) = &document.lint_errors {
            return Some(errors.clone());
        }
        let sender = &self.connection.sender;
        let engine = self.engines.get_or_build(config, &mut |warning| send_show(sender, MessageType::WARNING, warning));
        match lint(engine, &code(document.path.as_deref(), &document.text)) {
            Ok(errors) => {
                self.documents.get_mut(uri)?.lint_errors = Some(errors.clone());
                Some(errors)
            }
            Err(Failure::Parse) => None,
            Err(Failure::Other(message)) => {
                self.log(MessageType::ERROR, message);
                None
            }
        }
    }

    fn publish_diagnostics(&mut self, uri: &Uri) {
        let Some(path) = self.documents.get(uri).map(|d| d.path.clone()) else { return };
        let effective = self.effective(path.as_deref());
        let errors = self.lint_errors(uri, &effective).unwrap_or_default();
        let Some(document) = self.documents.get(uri) else { return };
        let diagnostics = diagnostics(&document.text, path.as_deref(), &errors, effective.unfixable_as_error);
        self.send_diagnostics(uri.clone(), diagnostics, Some(document.version));
    }

    fn formatting(&mut self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let uri = params.text_document.uri;
        let path = self.documents.get(&uri)?.path.clone();
        let effective = self.effective(path.as_deref());
        let document = self.documents.get(&uri)?;
        let name = path.as_ref().map_or_else(|| uri.as_str().to_owned(), |p| p.display().to_string());
        let formatted = match effective.formatter? {
            Formatter::Ktfmt { settings, editorconfig } => ktfmt(&document.text, &ktfmt_options(&settings, editorconfig, path.as_deref()), &name),
            Formatter::Ktlint(config) => {
                let sender = &self.connection.sender;
                let engine = self.engines.get_or_build(&config, &mut |warning| send_show(sender, MessageType::WARNING, warning));
                fix_all(engine, &code(path.as_deref(), &document.text)).map_err(|failure| match failure {
                    Failure::Parse => format!("{name}: not formatted, not valid Kotlin"),
                    Failure::Other(message) => message,
                })
            }
        };
        match formatted {
            Ok(formatted) => Some(text_edits(&document.text, &formatted)),
            Err(message) => {
                self.log(MessageType::WARNING, message);
                None
            }
        }
    }

    fn code_actions(&mut self, params: CodeActionParams) -> Vec<lsp_types::CodeActionOrCommand> {
        let uri = params.text_document.uri;
        let Some(path) = self.documents.get(&uri).map(|d| d.path.clone()) else { return Vec::new() };
        let effective = self.effective(path.as_deref());
        let Some(errors) = self.lint_errors(&uri, &effective) else { return Vec::new() };
        let (Some(config), Some(document)) = (effective.ktlint.as_ref(), self.documents.get(&uri)) else { return Vec::new() };
        let diagnostics = diagnostics(&document.text, path.as_deref(), &errors, effective.unfixable_as_error);
        let code = code(path.as_deref(), &document.text);
        let sender = &self.connection.sender;
        let engine = self.engines.get_or_build(config, &mut |warning| send_show(sender, MessageType::WARNING, warning));
        let linted = Linted { uri: &uri, engine, code: &code, errors: &errors, diagnostics: &diagnostics };
        code_actions(&linted, params.range, params.context.only.as_deref())
    }

    fn send_diagnostics(&self, uri: Uri, diagnostics: Vec<lsp_types::Diagnostic>, version: Option<i32>) {
        let params = PublishDiagnosticsParams { uri, diagnostics, version };
        self.send(Notification::new(n::PublishDiagnostics::METHOD.to_owned(), params).into());
    }

    fn log(&self, typ: MessageType, message: String) {
        send_log(&self.connection.sender, typ, message);
    }

    fn show(&self, typ: MessageType, message: String) {
        send_show(&self.connection.sender, typ, message);
    }

    fn send(&self, message: Message) {
        // Fails only once the client is gone, which ends the loop.
        let _ = self.connection.sender.send(message);
    }
}

fn send_log(sender: &Sender<Message>, typ: MessageType, message: String) {
    let _ = sender.send(Notification::new(n::LogMessage::METHOD.to_owned(), LogMessageParams { typ, message }).into());
}

fn send_show(sender: &Sender<Message>, typ: MessageType, message: String) {
    let _ = sender.send(Notification::new(n::ShowMessage::METHOD.to_owned(), ShowMessageParams { typ, message }).into());
}

fn params<P: DeserializeOwned>(value: Value) -> Result<P, String> {
    serde_json::from_value(value).map_err(|e| format!("invalid params: {e}"))
}

fn to_value(value: impl serde::Serialize) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}
