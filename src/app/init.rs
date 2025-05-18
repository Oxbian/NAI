use crate::app::llm::{Message, MessageType, LLM};
use crate::app::modules::{wikipedia, resume, chat, code};
use crate::helper::init::warn;
use uuid::Uuid;
use tokio::runtime::Builder;

pub struct App {
    pub messages: Vec<Message>, // History of recorded message
    pub conv_id: Uuid, // ID for retrieving and saving the history of messag
    categorize_llm: LLM,
}

impl App {
    pub fn new() -> App {

        App {
            messages: Vec::new(),
            conv_id: Uuid::new_v4(),
            categorize_llm: LLM::new("config/categorize-LLM.json"),
        }
    }

    fn append_message(&mut self, msg: String, role: MessageType) {
        let message = Message::new(role, msg);

        let err = message.save_message(self.conv_id.to_string());
        //warn(err.is_err().to_string()); 

        self.messages.push(message);
    }

    fn categorize_ask(&mut self) {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();

        let result = runtime.block_on(async {
            // Ask the LLM to categorise the request between (chat, code, wikipedia)
            self.categorize_llm.ask_tools(&self.messages).await
        });

        match result {
            Ok(msg) => {
                let categorie = msg[0]["function"]["arguments"]["category_choice"].clone();
                self.ask(&categorie.to_string().replace("\"", ""));
            },
            Err(e) => self.append_message(e.to_string(), MessageType::ASSISTANT),
        }
    }

    fn ask(&mut self, mode: &str) {
        warn(format!("Categorie: {}", mode));
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build().unwrap();

        let result = runtime.block_on(async {
            if mode == "resume" {
                resume::resume_conv(self.messages.clone()).await
            } else if mode == "wikipedia" {
                wikipedia::ask_wiki(&self.messages).await
            } else {
                chat::ask_chat(self.messages.clone()).await
            }
        });

        match result {
            Ok(msg) => self.append_message(msg.to_string(), MessageType::ASSISTANT),
            Err(e) => self.append_message(e.to_string(), MessageType::ASSISTANT),
        }
    }

    pub fn send_message(&mut self, content: String) {
        self.append_message(content, MessageType::USER);
        self.categorize_ask();
    }

    pub fn resume_conv(&mut self) {
        self.ask("resume"); 
    }
}
