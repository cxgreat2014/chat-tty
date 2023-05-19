use futures::stream::StreamExt;
use std::{fmt::Display, str::FromStr};

use eventsource_stream::Eventsource;
use futures::Stream;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::errors::Errors;

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageJson {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Payload {
    pub model: String,
    pub messages: Vec<MessageJson>,
    pub stream: bool,
}

pub struct Client {
    pub base: reqwest::Url,
    pub client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Role {
    /// A system message, automatically sent at the start to set the tone of the model
    System,
    // A message sent by ChatGPT
    Assistant,
    /// A message sent by the user
    User,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::System => write!(f, "system"),
        }
    }
}

impl FromStr for Role {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            _ => Err(Errors::InvalidRoleName),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InboundResponseChunk {
    /// All message chunks in this response part (only one usually)
    pub choices: Vec<InboundChunkChoice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InboundChunkChoice {
    /// The part value of the response
    pub delta: InboundChunkPayload,
    /// Index of the message this chunk refers to
    pub index: usize,
}

/// Contains different chunked inbound response payloads
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum InboundChunkPayload {
    /// Begins a single message by announcing roles (usually `assistant`)
    AnnounceRoles {
        /// The announced role
        role: String,
    },
    /// Streams a part of message content
    StreamContent {
        /// The part of content
        content: String,
    },
    /// Closes a single message
    Close {},
}

#[derive(Debug)]
pub enum Response {
    Content {
        delta: String,
        response_index: usize,
    },
    BeginResponse {
        role: Role,
        response_index: usize,
    },
    CloseResponse {
        response_index: usize,
    },
    Done,
}

impl Client {
    pub fn new(base: &str, api_key: &str) -> anyhow::Result<Self> {
        let mut headers = header::HeaderMap::new();
        let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", api_key))?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let base = reqwest::Url::parse(base)?;

        Ok(Self { client, base })
    }

    pub async fn complete_chat(
        &self,
        messages: Vec<MessageJson>,
    ) -> anyhow::Result<impl Stream<Item = Response>> {
        let payload = Payload {
            model: "gpt-4".into(),
            messages,
            stream: true,
        };

        let url = self.base.join("/v1/chat/completions")?;

        let res = self.client.post(url).json(&payload).send().await?;
        if res.status() != 200 {
            return Err(anyhow::anyhow!("Status: {}", res.status()));
        }
        let bytes = res.bytes_stream();

        let stream = bytes.eventsource().map(move |part| -> Response {
            let chunk = &part.expect("Stream closed abruptly").data;

            if chunk == "[DONE]" {
                return Response::Done;
            }
            let data: InboundResponseChunk = serde_json::from_str(chunk).expect("Invalid JSON");
            let choice = data
                .choices
                .first()
                .expect("No choices in response")
                .to_owned();

            match choice.delta {
                InboundChunkPayload::AnnounceRoles { role } => Response::BeginResponse {
                    role: Role::from_str(&role).expect("Invalid role"),
                    response_index: choice.index,
                },
                InboundChunkPayload::StreamContent { content } => Response::Content {
                    delta: content,
                    response_index: choice.index,
                },
                InboundChunkPayload::Close {} => Response::CloseResponse {
                    response_index: choice.index,
                },
            }
        });

        Ok(stream)
    }
}
