use crate::helper::init::print_in_file;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct LLM {
    url: String,
    model: String,
    pub system_prompt: String,
}

impl LLM {
    pub fn new(config_file: String) -> Result<LLM, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(config_file)?;
        let llm: LLM = serde_json::from_str(&contents)?;

        Ok(llm)
    }

    pub async fn ask(&self, messages: &Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .json(&serde_json::json!({
               "model": self.model,
               "messages": messages,
               "stream": true}))
            .send()
            .await?;

        let mut full_message = String::new();

        // Reading the stream and saving the response
        match response.error_for_status() {
            Ok(mut res) => {
                while let Some(chunk) = res.chunk().await? {
                    let answer: Value = serde_json::from_slice(chunk.as_ref())?;

                    print_in_file(answer.to_string());
                    if answer["done"].as_bool().unwrap_or(false) {
                        break;
                    }

                    let msg = answer["message"]["content"].as_str().unwrap_or("\n");

                    full_message.push_str(msg);
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    
        print_in_file(full_message.clone());
        Ok(full_message)
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum MessageType {
    ASSISTANT,
    SYSTEM,
    USER,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::ASSISTANT => write!(f, "assistant"),
            MessageType::SYSTEM => write!(f, "system"),
            MessageType::USER => write!(f, "user"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    role: MessageType,
    content: String,
}

impl Message {
    pub fn new(role: MessageType, content: String) -> Message {
        Message { role, content }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.role {
            MessageType::USER => return write!(f, "You: {}", self.content),
            MessageType::SYSTEM => return write!(f, "System: {}", self.content),
            MessageType::ASSISTANT => return write!(f, "Néo AI: {}", self.content),
        }
    }
}
