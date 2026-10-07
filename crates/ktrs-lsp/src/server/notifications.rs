//! The server's notification handlers: document sync, settings and watched files.

use lsp_server::{Notification, Request, RequestId};
use lsp_types::notification::{self as n, Notification as _};
use lsp_types::request::{self as r, Request as _};
use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidChangeWatchedFilesRegistrationOptions,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams, FileChangeType, FileSystemWatcher, GlobPattern,
    MessageType, Registration, RegistrationParams,
};
use serde_json::Value;

use super::{Server, params};
use crate::documents::uri_to_path;
use crate::settings::Settings;

impl Server<'_> {
    pub(super) fn notify(&mut self, notification: Notification) {
        let Notification { method, params: value } = notification;
        let result = match method.as_str() {
            n::DidOpenTextDocument::METHOD => params(value).map(|p: DidOpenTextDocumentParams| {
                let document = p.text_document;
                self.documents.open(document.uri, document.text, document.version);
            }),
            n::DidChangeTextDocument::METHOD => params(value).map(|p: DidChangeTextDocumentParams| {
                if let Some(change) = p.content_changes.into_iter().last() {
                    self.documents.change(&p.text_document.uri, change.text, p.text_document.version);
                }
            }),
            n::DidSaveTextDocument::METHOD => params(value).map(|p: DidSaveTextDocumentParams| {
                if let Some(document) = self.documents.get_mut(&p.text_document.uri) {
                    document.lint_errors = None;
                    self.documents.mark_stale(p.text_document.uri);
                }
            }),
            n::DidCloseTextDocument::METHOD => params(value).map(|p: DidCloseTextDocumentParams| {
                self.documents.close(&p.text_document.uri);
                self.send_diagnostics(p.text_document.uri, Vec::new(), None);
            }),
            n::DidChangeConfiguration::METHOD => params(value).map(|p: DidChangeConfigurationParams| self.configure(&p.settings)),
            n::DidChangeWatchedFiles::METHOD => params(value).map(|p: DidChangeWatchedFilesParams| self.files_changed(p)),
            _ => Ok(()),
        };
        if let Err(message) = result {
            self.log(MessageType::ERROR, format!("{method}: {message}"));
        }
    }

    /// Replaces the settings with `value`'s (kept on an invalid value) and marks every document for re-linting.
    pub(super) fn configure(&mut self, value: &Value) {
        match Settings::parse(value) {
            Ok(settings) => {
                self.settings = settings;
                self.documents.invalidate_all();
            }
            Err(message) => self.show(MessageType::ERROR, message),
        }
    }

    fn files_changed(&mut self, params: DidChangeWatchedFilesParams) {
        let mut changed = false;
        for event in params.changes {
            let Some(path) = uri_to_path(&event.uri) else { continue };
            if path.file_name().is_some_and(|name| name == ".editorconfig") {
                if let Err(message) = self.engines.editor_config_changed(&path, event.typ != FileChangeType::CHANGED) {
                    self.log(MessageType::ERROR, format!("{}: {message}", path.display()));
                }
                changed = true;
            }
            if ktrs_project::is_build_file(&path) {
                ktrs_project::invalidate(&path);
                changed = true;
            }
        }
        if changed {
            self.documents.invalidate_all();
        }
    }

    /// Asks the client to watch `.editorconfig` and build files (filtered by `ktrs_project::is_build_file`).
    pub(super) fn register_watchers(&self) {
        let globs = ["**/.editorconfig", "**/*.gradle", "**/*.gradle.kts", "**/pom.xml", "**/*.toml", "**/*.properties"];
        let watchers = globs.map(|glob| FileSystemWatcher { glob_pattern: GlobPattern::String(glob.to_owned()), kind: None }).to_vec();
        let registration = Registration {
            id: "ktrs-watched-files".to_owned(),
            method: n::DidChangeWatchedFiles::METHOD.to_owned(),
            register_options: Some(serde_json::to_value(DidChangeWatchedFilesRegistrationOptions { watchers }).expect("serializable")),
        };
        let params = RegistrationParams { registrations: vec![registration] };
        let request = Request::new(RequestId::from("ktrs-register-watchers".to_owned()), r::RegisterCapability::METHOD.to_owned(), params);
        self.send(request.into());
    }
}
