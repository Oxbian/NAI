use crate::helper::init::warn;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fs::{self, OpenOptions, create_dir_all};
use std::io::Write;

#[derive(Deserialize, Debug)]
pub struct LLM {
    url: String,
    model: String,
    pub system_prompt: String,
    pub tools: serde_json::Value,
}

impl LLM {
    pub fn new(config_file: &str) -> LLM {
        let contents = fs::read_to_string(config_file).unwrap();
        serde_json::from_str(&contents).unwrap()
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

                    //warn(answer.to_string());
                    if answer["done"].as_bool().unwrap_or(false) {
                        break;
                    }

                    let msg = answer["message"]["content"].as_str().unwrap_or("\n");

                    full_message.push_str(msg);
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    
        warn(full_message.clone());
        Ok(full_message)
    }

    // Use tools functionnality of Ollama, only some models supports it:
    // https://ollama.com/search?c=tools
    pub async fn ask_tools(&self, messages: &Vec<Message>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "stream": false,
                "tools": self.tools}))
            .send()
            .await?.json::<Value>().await?;

        //warn(response.to_string());

        if let Some(tool_calls) = response
            .get("message")
                .and_then(|msg| msg.get("tool_calls"))
                .cloned()
        {
            Ok(tool_calls)
        } else {
            Err("tool_calls not found".into())
        }
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
    pub role: MessageType,
    pub content: String,
}

impl Message {
    pub fn new(role: MessageType, content: String) -> Message {
        Message { role, content }
    }

    pub fn default() -> Message {
        Message {
            role: MessageType::USER,
            content: "".to_string(),
        }
    }

    pub fn save_message(&self, conv_id: String) -> Result<(), Box<dyn std::error::Error>> {
        // Create conv directory if doesn't exist
        create_dir_all("conv")?;

        // Save message
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("conv/".to_string() + &conv_id)
            .unwrap();

        writeln!(file, "{}", serde_json::to_string(self)?)?;

        Ok(())
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
