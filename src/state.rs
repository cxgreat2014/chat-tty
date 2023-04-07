use colored::Colorize;
use std::io::Write;
use futures::stream::StreamExt;
use std::{time::{SystemTime, UNIX_EPOCH}, pin::Pin};
use futures::Stream;

use crate::{client::{Role, Response}, client::{Client, MessageJson}};

#[non_exhaustive]
pub struct AppState {
    client: Option<Client>,
    messages: Messages,
}


struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    fn to_payload(&self) -> Vec<MessageJson> {
        self.messages.iter().filter_map(|m| {
            match m {
                Message::InProgress(_) => None,
                Message::Complete(m) => Some(MessageJson{
                    content: m.text.clone(),
                    role: m.direction.to_string()
                })
            }
        }).collect()
    }
}


impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    async fn append_from_stream(&mut self, s: &mut Pin<&mut impl Stream<Item = Response>>) {
        while let Some(x) = s.next().await {
            // dbg!(&x);
            match x {
                Response::BeginResponse { role, response_index: _ } => {
                    self.messages.push(Message::InProgress(IncompleteMessage {
                        direction: role,
                        partial_text: "".into()
                    }));
                },
                Response::Content { delta, response_index: _ } => {
                    let last_idx = self.messages.len() - 1;
                    let end = self.messages.get_mut(last_idx).unwrap();
                    print!("{}", delta.cyan());
                    std::io::stdout().flush().unwrap();
                    
                    // dbg!(&delta);
                    match end {
                        Message::Complete(_) => panic!("Attempted to append to completed message"),
                        Message::InProgress(m) => {
                            m.partial_text.push_str(&delta);
                        }
                    }
                },
                Response::Done => {
                    let last_idx = self.messages.len() - 1;
                    let end = self.messages.get_mut(last_idx).unwrap();
                    let complete: CompleteMessage = end.clone().into();
                    *end = Message::Complete(complete);
                    println!("\n");
                },
                _ => {}
            }
        }
        // for r in s {
        //     match r {
        //         Response::BeginResponse { role, response_index } => {

        //         }
        //     }
        //     self.messages.push(message);
        // }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Complete(CompleteMessage),
    InProgress(IncompleteMessage),
}

#[derive(Debug, Clone)]
struct CompleteMessage {
        direction: Role,
        text: String,
        time: i32
}

#[derive(Debug, Clone)]
struct IncompleteMessage {
        direction: Role,
        partial_text: String,
}

impl From<Message> for CompleteMessage {
    fn from(m: Message) -> Self {
        match m {
            Message::Complete(c) => c,
            Message::InProgress(i) => Self {
            direction: i.direction,
            text: i.partial_text,
            time: time(),
        }
        }
        
    }
}

fn time() -> i32 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs() as i32

}


impl AppState {
    pub fn new(api_key: Option<&str>) -> Self {
        Self {
            client: api_key.and_then(|k| Client::new(k).ok()),
            messages: Messages::new(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.client.is_some()
    }

    pub fn set_api_key(&mut self, api_key: &str) {
        let client = Client::new(api_key).ok();
        self.client = client;
    }

    pub async fn add_message(&mut self, text: String) -> anyhow::Result<()> {
        let message = Message::Complete(CompleteMessage{
            direction: Role::User,
            text,
            time: time(),
        });
        self.messages.messages.push(message);
        self.sync_server_state().await
    }

    async fn sync_server_state(&mut self) -> anyhow::Result<()> {
        let payload = self.messages.to_payload();
        let c = self.client.as_mut().unwrap();
        let mut stream = c.complete_chat(payload).await?;
        let mut pin = Pin::new(&mut stream);
        self.messages.append_from_stream(&mut pin).await;
        Ok(())
    }
}
