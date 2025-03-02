use crate::app::llm::{Message, MessageType, LLM};
use crate::helper::init::print_in_file;
use tokio;

pub struct App {
    pub messages: Vec<Message>, // History of recorded message
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
            chat_llm,
            resume_llm: LLM::new("config/resume-LLM.json".to_string()).unwrap(),
        }
    }

    fn append_message(&mut self, msg: String, role: MessageType) {
        let message = Message::new(role, msg);
        self.messages.push(message);
    }

    pub fn send_message(&mut self, content: String) {
        self.append_message(content, MessageType::USER);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        let result = runtime.block_on(async {
            self.chat_llm.ask(&self.messages).await
        });

        match result {
            Ok(msg) => self.append_message(msg, MessageType::ASSISTANT),
            Err(e) => self.append_message(e.to_string(), MessageType::ASSISTANT),
        }
    }

    pub fn resume_conv(&mut self) {
        self.append_message(self.resume_llm.system_prompt.to_string(), MessageType::USER);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        
        let result = runtime.block_on(async {
            self.resume_llm.ask(&self.messages).await
        });

        match result {
            Ok(msg) => self.append_message(msg, MessageType::ASSISTANT),
            Err(e) => self.append_message(e.to_string(), MessageType::ASSISTANT),
        }
    }
}
