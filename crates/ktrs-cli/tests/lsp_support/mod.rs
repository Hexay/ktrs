#![allow(dead_code)]
//! An in-process LSP client for `ktrs lsp`: the server runs on a thread over lsp-server's in-memory connection.

use std::path::Path;
use std::thread::JoinHandle;
use std::time::Duration;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{Diagnostic, PublishDiagnosticsParams, TextEdit, Uri};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(60);

pub struct Client {
    connection: Connection,
    server: Option<JoinHandle<Result<(), String>>>,
    next_id: i32,
    /// `window/logMessage` and `window/showMessage` texts, in order.
    pub logs: Vec<String>,
    pub shown: Vec<String>,
    diagnostics: Vec<PublishDiagnosticsParams>,
}

impl Client {
    pub fn start(initialization_options: Value) -> Client {
        let (server, connection) = Connection::memory();
        let server = std::thread::spawn(move || ktrs_lsp::serve(server));
        let mut client = Client { connection, server: Some(server), next_id: 0, logs: Vec::new(), shown: Vec::new(), diagnostics: Vec::new() };
        let capabilities = json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}});
        let result = client.request("initialize", json!({"capabilities": capabilities, "initializationOptions": initialization_options}));
        assert_eq!(result["serverInfo"]["name"], "ktrs");
        client.notify("initialized", json!({}));
        client
    }

    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = RequestId::from(self.next_id);
        self.connection.sender.send(Request::new(id.clone(), method.to_owned(), params).into()).unwrap();
        loop {
            match self.receive() {
                Message::Response(Response { id: response_id, response_result }) if response_id == id => {
                    return response_result.unwrap_or_else(|error| panic!("{method}: {error:?}"));
                }
                message => self.record(message),
            }
        }
    }

    pub fn notify(&self, method: &str, params: Value) {
        self.connection.sender.send(Notification::new(method.to_owned(), params).into()).unwrap();
    }

    /// The next diagnostics published for `uri` (with `version`, when given; earlier ones are skipped).
    pub fn diagnostics(&mut self, uri: &Uri, version: Option<i32>) -> Vec<Diagnostic> {
        loop {
            let wanted = |p: &PublishDiagnosticsParams| p.uri == *uri && (version.is_none() || p.version == version);
            if let Some(i) = self.diagnostics.iter().position(wanted) {
                return self.diagnostics.drain(..=i).last().unwrap().diagnostics;
            }
            let message = self.receive();
            self.record(message);
        }
    }

    pub fn open(&mut self, path: &Path, text: &str) -> Uri {
        let uri = ktrs_lsp::path_to_uri(path);
        let document = json!({"uri": uri, "languageId": "kotlin", "version": 1, "text": text});
        self.notify("textDocument/didOpen", json!({"textDocument": document}));
        uri
    }

    pub fn change(&self, uri: &Uri, version: i32, text: &str) {
        let params = json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]});
        self.notify("textDocument/didChange", params);
    }

    pub fn format(&mut self, uri: &Uri) -> Option<Vec<TextEdit>> {
        let params = json!({"textDocument": {"uri": uri}, "options": {"tabSize": 4, "insertSpaces": true}});
        serde_json::from_value(self.request("textDocument/formatting", params)).unwrap()
    }

    pub fn format_range(&mut self, uri: &Uri, range: lsp_types::Range) -> Option<Vec<TextEdit>> {
        let params = json!({"textDocument": {"uri": uri}, "range": range, "options": {"tabSize": 4, "insertSpaces": true}});
        serde_json::from_value(self.request("textDocument/rangeFormatting", params)).unwrap()
    }

    /// The code actions over the whole document as (title, kind, edits for `uri`).
    pub fn code_actions(&mut self, uri: &Uri, only: Option<&[&str]>) -> Vec<(String, String, Vec<TextEdit>)> {
        let range = json!({"start": {"line": 0, "character": 0}, "end": {"line": 10000, "character": 0}});
        let context = match only {
            Some(only) => json!({"diagnostics": [], "only": only}),
            None => json!({"diagnostics": []}),
        };
        let actions = self.request("textDocument/codeAction", json!({"textDocument": {"uri": uri}, "range": range, "context": context}));
        let actions: Vec<lsp_types::CodeAction> = serde_json::from_value(actions).unwrap();
        actions
            .into_iter()
            .map(|a| {
                let edits = a.edit.unwrap().changes.unwrap().remove(uri).unwrap();
                (a.title, a.kind.unwrap().as_str().to_owned(), edits)
            })
            .collect()
    }

    pub fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.server.take().unwrap().join().unwrap().unwrap();
    }

    fn receive(&self) -> Message {
        self.connection.receiver.recv_timeout(TIMEOUT).expect("the server answers")
    }

    fn record(&mut self, message: Message) {
        match message {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                self.diagnostics.push(serde_json::from_value(n.params).unwrap());
            }
            Message::Notification(n) if n.method == "window/logMessage" => self.logs.push(n.params["message"].as_str().unwrap().to_owned()),
            Message::Notification(n) if n.method == "window/showMessage" => self.shown.push(n.params["message"].as_str().unwrap().to_owned()),
            Message::Request(request) => {
                assert_eq!(request.method, "client/registerCapability");
                self.connection.sender.send(Response::new_ok(request.id, Value::Null).into()).unwrap();
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
