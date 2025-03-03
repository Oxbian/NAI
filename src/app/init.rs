use crate::app::llm::{Message, MessageType, LLM};
use crate::helper::init::warn;
use uuid::Uuid;
use tokio::runtime::Builder;

pub struct App {
    pub messages: Vec<Message>, // History of recorded message
    conv_id: Uuid,
    chat_llm: LLM,
    resume_llm: LLM,
}

impl App {
    pub fn new() -> App {
        let chat_llm: LLM = LLM::new("config/chat-LLM.json".to_string()).unwrap();
        App {
            messages: vec![Message::new(
                MessageType::SYSTEM,
                chat_llm.system_prompt.clone(),
            )],
            conv_id: Uuid::new_v4(),
            chat_llm,
            resume_llm: LLM::new("config/resume-LLM.json".to_string()).unwrap(),
        }
    }

    fn append_message(&mut self, msg: String, role: MessageType) {
        let message = Message::new(role, msg);
        message.save_message(self.conv_id.to_string());
        self.messages.push(message);
    }

    fn ask(&mut self, mode: &str) {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        let result = runtime.block_on(async {
            if mode == "resume" {
                self.resume_llm.ask(&self.messages).await
            } else {
                self.chat_llm.ask(&self.messages).await
            }
        });

        match result {
            Ok(msg) => self.append_message(msg, MessageType::ASSISTANT),
            Err(e) => self.append_message(e.to_string(), MessageType::ASSISTANT),
        }
    }

    pub fn send_message(&mut self, content: String) {
        self.append_message(content, MessageType::USER);
        self.ask("chat");
    }

    pub fn resume_conv(&mut self) {
        self.append_message(self.resume_llm.system_prompt.to_string(), MessageType::USER);
        self.ask("resume"); 
    }
}
