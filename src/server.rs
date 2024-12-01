use crossbeam_channel::Sender;

use lsp_server::{Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, Notification as NotificationMessage,
    },
    request::{Completion, GotoDefinition, HoverRequest, Request as RequestMessage},
    CompletionItem, CompletionParams, CompletionResponse, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, Location, MarkedString, Position,
    Uri,
};

use anyhow::Result;

use crate::{
    instructions,
    symbol_cache::{symbol_cache_fetch, symbol_cache_insert, symbol_cache_reset},
    text_store::{self, get_text_document, TEXT_STORE},
};

fn cast_req<R>(req: Request) -> Result<(RequestId, R::Params)>
where
    R: RequestMessage,
    R::Params: serde::de::DeserializeOwned,
{
    match req.extract(R::METHOD) {
        Ok(value) => Ok(value),
        Err(e) => Err(anyhow::anyhow!("Error: {e}")),
    }
}

fn cast_notif<N>(notif: Notification) -> Result<N::Params>
where
    N: NotificationMessage,
    N::Params: serde::de::DeserializeOwned,
{
    match notif.extract(N::METHOD) {
        Ok(value) => Ok(value),
        Err(e) => Err(anyhow::anyhow!("Error: {e}")),
    }
}

pub struct Server {
    sender: Sender<Message>,
}

impl Server {
    pub fn new(sender: Sender<Message>) -> Server {
        Server { sender }
    }

    pub fn dispatch(&self, message: Message) -> Result<()> {
        tracing::info!("Dispatching");
        match message {
            Message::Notification(notification) => self.dispatch_notification(notification),
            Message::Request(request) => self.dispatch_request(request),
            _ => unreachable!("This shouldn't be able to happen"),
        }
    }

    fn dispatch_notification(&self, notification: Notification) -> Result<()> {
        // tracing::error!("Notification: {}", notification.method);
        match notification.method.as_str() {
            DidChangeTextDocument::METHOD => {
                let notification = cast_notif::<DidChangeTextDocument>(notification)?;
                let uri = notification.text_document.uri;
                let text = notification.content_changes[0].text.to_string();

                TEXT_STORE
                    .get()
                    .expect("TEXT_STORE not initialized")
                    .lock()
                    .expect("text store mutex poisoned")
                    .insert(uri.to_string(), text);

                self.parse_labels(&uri);
            }
            DidOpenTextDocument::METHOD => {
                let notification = cast_notif::<DidOpenTextDocument>(notification)?;

                // tracing::info!("{:#?}", notification);

                let uri = notification.text_document.uri;
                let text = notification.text_document.text;

                TEXT_STORE
                    .get()
                    .expect("TEXT_STORE not initialized")
                    .lock()
                    .expect("text store mutex poisoned")
                    .insert(uri.to_string(), text);

                self.parse_labels(&uri);
            }
            _ => {}
        }

        Ok(())
    }

    fn dispatch_request(&self, request: Request) -> Result<()> {
        tracing::error!("Request: {}", request.method);
        match request.method.as_str() {
            Completion::METHOD => {
                let (id, params) = cast_req::<Completion>(request)?;

                let result = self.completion(&params);

                let result = serde_json::to_value(result).unwrap();
                let result = Response {
                    id,
                    result: Some(result),
                    error: None,
                };

                self.sender.send(Message::Response(result))?
            }
            HoverRequest::METHOD => {
                let (id, params) = cast_req::<HoverRequest>(request)?;

                let result = self.hover(&params)?;
                let result = serde_json::to_value(result).unwrap();

                let result = Response {
                    id,
                    result: Some(result),
                    error: None,
                };

                self.sender.send(Message::Response(result))?
            }
            GotoDefinition::METHOD => {
                let (id, params) = cast_req::<GotoDefinition>(request)?;

                let result = self.goto_definition(&params)?;
                let result = serde_json::to_value(result).unwrap();

                let result = Response {
                    id,
                    result: Some(result),
                    error: None,
                };

                self.sender.send(Message::Response(result))?
            }
            _ => {}
        }

        Ok(())
    }

    fn completion(&self, _params: &CompletionParams) -> Option<CompletionResponse> {
        Some(CompletionResponse::Array(vec![CompletionItem::new_simple(
            "test".to_string(),
            "cool".to_string(),
        )]))
    }

    fn hover(&self, params: &HoverParams) -> Result<Option<Hover>> {
        let result = text_store::get_word_from_pos_params(&params.text_document_position_params)?;

        let is_6502_opcode = result.len() == 3;
        if is_6502_opcode {
            let candidate = instructions::INSTRUCTION_MAP
                .get()
                .expect("Instructions not loaded")
                .get(&result.to_uppercase());
            if let Some(docs) = candidate {
                let hover = Hover {
                    contents: HoverContents::Scalar(MarkedString::from_markdown(docs.clone())),
                    range: None,
                };

                return Ok(Some(hover));
            }
        } else {
            let hover = Hover {
                contents: HoverContents::Scalar(MarkedString::from_markdown(result.to_string())),
                range: None,
            };

            return Ok(Some(hover));
        }

        Ok(None)
    }

    fn goto_definition(
        &self,
        params: &GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let result = text_store::get_word_from_pos_params(&params.text_document_position_params)?;

        let symbol = symbol_cache_fetch(result);

        if let Some(symbol) = symbol {
            let start = Position::new(symbol.line as u32, 0);
            let end = Position::new(symbol.line as u32, 1);
            return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                symbol.uri,
                lsp_types::Range::new(start, end),
            ))));
        }

        Ok(None)
    }

    fn parse_labels(&self, uri: &Uri) {
        let document = get_text_document(uri).expect("File not open??");
        symbol_cache_reset(uri);

        for (i, line) in document.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            let split: Vec<&str> = line.split_whitespace().collect();

            if split.len() > 0 {
                let maybe_ident = split[0];
                if is_identifier(maybe_ident)
                    && instructions::INSTRUCTION_MAP
                        .get()
                        .expect("Instructions not loaded")
                        .get(&maybe_ident.to_uppercase())
                        .is_none()
                {
                    let parsed_ident = parse_ident(maybe_ident);
                    symbol_cache_insert(uri, i, parsed_ident.clone());
                    // tracing::info!("Ident {}", parsed_ident.clone());
                }
            }
        }
    }
}

fn parse_ident(ident: &str) -> String {
    if ident.ends_with(":") {
        return ident[0..ident.len() - 1].to_string();
    }

    ident.to_string()
}

fn is_identifier(ident: &str) -> bool {
    for (i, char) in ident.as_bytes().iter().enumerate() {
        if !(char.is_ascii_alphanumeric() || *char == b'_') {
            if *char == b':' && i == ident.len() - 1 {
                return true;
            }

            return false;
        }
    }
    true
}
