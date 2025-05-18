use crate::app::llm::{LLM, Message, MessageType};

pub async fn ask_chat(mut messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    let chat_llm = LLM::new("config/chat-LLM.json");
    messages.push(Message::new(MessageType::USER, chat_llm.system_prompt.to_string()));
  
    let result: String = chat_llm.ask(&messages).await?;
    Ok(result)
}
