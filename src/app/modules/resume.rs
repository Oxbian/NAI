use crate::app::llm::{LLM, Message, MessageType};

pub async fn resume_conv(mut messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    let resume_llm = LLM::new("config/resume-LLM.json");
    messages.push(Message::new(MessageType::USER, resume_llm.system_prompt.to_string()));
  
    let result: String = resume_llm.ask(&messages).await?;
    Ok(result)
}
