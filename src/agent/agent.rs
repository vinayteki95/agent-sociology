use std::collections::VecDeque;

use tokio::sync::mpsc;

use super::errors::AgentError;
use crate::agent::message::{Message, Role};

const MODEL_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_MAX_MEMORY: usize = 10;

#[derive(serde::Deserialize)]
struct OllamaResponse {
    // response: String,
    // thinking: Option<String>,
    message: Message,
}

pub struct Agent {
    _id: String,
    model: String,
    httpclient: reqwest::Client,
    messages: VecDeque<Message>,
    system_prompt: String,
    max_memory: usize,
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<String>,
}

impl Agent {
    pub fn new(
        id: &str,
        model: &str,
        system_prompt: &str,
        rx: mpsc::Receiver<String>,
        tx: mpsc::Sender<String>,
    ) -> Self {
        Self {
            _id: id.to_string(),
            model: model.to_string(),
            httpclient: reqwest::Client::new(),
            messages: VecDeque::with_capacity(DEFAULT_MAX_MEMORY + 2),
            system_prompt: system_prompt.to_string(),
            max_memory: DEFAULT_MAX_MEMORY,
            rx,
            tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match self.call_ollama(&msg).await {
                Ok(response) => {
                    let _ = self.tx.send(response).await;
                }
                Err(err) => panic!("Agent communication failed, {}", err),
            }
        }
    }

    pub fn set_max_memory(&mut self, max_memory: usize) {
        self.max_memory = max_memory;
    }

    fn append_message(&mut self, role: Role, content: &str) {
        self.messages.push_back(Message::new(role, content));
        while self.messages.len() > self.max_memory {
            self.messages.pop_front();
        }
    }

    pub async fn call_ollama(&mut self, message: &str) -> Result<String, AgentError> {
        self.append_message(Role::User, message);

        let system_message = Message::new(Role::System, &self.system_prompt);

        let messages: Vec<&Message> = std::iter::once(&system_message)
            .chain(self.messages.iter())
            .collect();
        // println!("messages: {:?}", messages);

        let req_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
            "think": false
        });

        let res = self
            .httpclient
            .post(MODEL_URL)
            .json(&req_body)
            .send()
            .await?;

        let resp_body = res.text().await?;

        let value: OllamaResponse = serde_json::de::from_str(&resp_body)?;

        self.append_message(Role::Assistant, &value.message.content);
        println!("***{}: {}", self._id, &value.message.content);
        println!("------------------------------------------");
        Ok(value.message.content.clone())
    }
}

// Tests ------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sliding_window() {
        let (tx, rx) = mpsc::channel(10);
        let mut agent = Agent::new("1", "model", "system", rx, tx);
        agent.set_max_memory(3);
        agent.append_message(Role::User, "msg1");
        agent.append_message(Role::User, "msg2");
        agent.append_message(Role::User, "msg3");
        agent.append_message(Role::User, "msg4");

        assert!(agent.messages.len() <= agent.max_memory);
        assert_eq!(agent.messages[0].content, "msg2");
        assert_eq!(agent.messages[1].content, "msg3");
        assert_eq!(agent.messages[2].content, "msg4");
    }
}
