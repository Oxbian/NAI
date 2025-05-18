use crate::app::llm::{Message, MessageType, LLM};
use crate::app::modules::wikipedia::ask_wiki;
use crate::helper::init::warn;
use uuid::Uuid;
use tokio::runtime::Builder;

pub struct App {
    pub messages: Vec<Message>, // History of recorded message
    pub conv_id: Uuid, // ID for retrieving and saving the history of messag
    categorize_llm: LLM,
    chat_llm: LLM, // Configuration for the LLM that chat with you
    resume_llm: LLM, // Configuration for the LLM that resume conversation
}

impl App {
    pub fn new() -> App {

        let categorize_llm = LLM::new("config/categorize-LLM.json");
        App {
            messages: vec![Message::new(
                MessageType::SYSTEM,
                categorize_llm.system_prompt.clone(),
            )],
            conv_id: Uuid::new_v4(),
            categorize_llm,
            chat_llm: LLM::new("config/chat-LLM.json"),
            resume_llm: LLM::new("config/resume-LLM.json"),
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
                let categorie = msg[0]["function"]["arguments"]["category"].clone();
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
                self.resume_llm.ask(&self.messages).await
            } else if mode == "wikipedia" {
                ask_wiki(&self.messages).await
            } else {
                self.chat_llm.ask(&self.messages).await
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
        self.append_message(self.resume_llm.system_prompt.to_string(), MessageType::USER);
        self.ask("resume"); 
    }
}
