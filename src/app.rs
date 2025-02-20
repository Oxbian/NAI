use color_eyre::Result;
use reqwest;
use serde_json::Value;
use std::{collections::HashMap, fmt};

#[derive(Debug)]
pub struct App {
    pub messages: Vec<Message>, // History of recorded messages
}

#[derive(Debug)]
pub struct Message {
    content: String,
    msg_type: MessageType,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.msg_type {
            MessageType::Human => return write!(f, "You: {}", self.content),
            MessageType::LLM => return write!(f, "Néo AI: {}", self.content),
        }
    }
}

#[derive(Debug)]
pub enum MessageType {
    Human,
    LLM,
}

impl App {
    pub fn new() -> App {
        App {
            messages: Vec::new(),
        }
    }

    pub fn send_message(&mut self, content: String) -> Result<()> {
        // POST: http://localhost:8080/completion {"prompt": "lorem ipsum"}
        self.messages.push(Message {
            content: content.clone(),
            msg_type: MessageType::Human,
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("http://localhost:8080/completion")
            .json(&serde_json::json!({
                "prompt": &content,
                "n_predict": 128,
            }))
            .send()?;

        if response.status().is_success() {
            // Désérialiser la réponse JSON
            let json_response: Value = response.json()?;

            // Accéder à la partie spécifique du JSON
            if let Some(msg) = json_response["content"].as_str() {
                self.messages.push(Message {
                    content: msg.to_string().clone(),
                    msg_type: MessageType::LLM,
                });
            } else {
                println!("Le champ 'data.id' est absent ou mal formaté.");
            }
        } else {
            eprintln!("La requête a échoué avec le statut : {}", response.status());
        }

        Ok(())
    }
}
