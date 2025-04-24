use std::sync::Arc;
use anyhow::Result;
use gtk::{prelude::TextBufferExt, glib};
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}, Credentials
};
use tokio::sync::mpsc::error::TryRecvError;
use crate::context::Context;
use tracing::{info, debug, error};
use ollama_rs::generation::completion::request::GenerationRequest;
use tokio_stream::StreamExt;
use crate::helper::convert_text;
use async_channel::Sender;
//use std::io::Write;

pub async fn ask_chat(ctx: Arc<Context>, sx: Sender<String>) -> Result<()> {
    let ai_chat = ctx.ai_chat.lock().await;
    let aic = ai_chat.clone();
    let wait = ctx.conf.chat_msg_wait;
    drop(ai_chat);
    let ai = match aic {
        Some(crate::AiChat::Grok) | Some(crate::AiChat::ChatGPT) | Some(crate::AiChat::Deepseek) => aic.unwrap(),
        _ => {
            error!("Invalid AI configuration");
            return Err(anyhow::anyhow!("Invalid AI configuration"))
        }
    };

    let ai_conf = match ai {
        crate::AiChat::Grok => ctx.conf.grok.clone().unwrap(),
        crate::AiChat::ChatGPT => ctx.conf.gpt.clone().unwrap(),
        crate::AiChat::Deepseek => ctx.conf.deepseek.clone().unwrap(),
        _ => {
            error!("Invalid AI configuration, it shouldn'e even be here");
            return Err(anyhow::anyhow!("Invalid AI configuration!!!"))
        }
    };

    info!("Config AI: {}", ai);
    let url = ai_conf.url;
    info!("Config URL: {}", url);
    let api_key = ai_conf.key;
    let model = ai_conf.model;
    let text_buffer = ctx.text_buffer().await;

    let prompt = crate::get_text!(text_buffer).to_string();

    debug!("Prompt: {}", prompt);
    let c = Credentials::new(api_key, url);

    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(prompt),
        name: None,
        function_call: None,
        tool_calls: None,
        tool_call_id: None,
    }];

    debug!("Created messages");
    let mut cc = ChatCompletion::builder(model.as_str(), messages.clone())
        .credentials(c.clone())
        .stream(true)
        .create_stream()
        .await?;

    debug!("Completions ready");

    let mut d = true;
    let dur = std::time::Duration::from_millis(wait);
    let result_buffer = ctx.result_buffer().await;
    let mut end_iter = result_buffer.end_iter();
    let mut vc = vec![];
    let play = ctx.with_sound.lock().await;

    while d {
        let r = cc.try_recv();
        match r {
            Ok(r) => {
                let choice = &r.choices[0];
                if let Some(content) = &choice.delta.content {
                    debug!("Received content: {}", content);
                    result_buffer.insert(&mut end_iter, content.as_str());
                    let has_dot = content.contains(".") || content.contains("。");
                    if *play {
                        vc.push(content.clone());
                        if has_dot && vc.len() > 10 {
                            match sx.send(vc.join(" ")).await {
                                Ok(_) => vc.clear(),
                                Err(e) => error!("Error sending: {}", e.to_string()),
                            }
                        }
                    }
                    
                } else {
                    debug!("I don't know what to do with it");
                }
            }
            Err(TryRecvError::Empty) => {
                debug!("Empty stream");
                glib::timeout_future(dur).await;
                debug!("Empty stream: awake");
            }
            Err(TryRecvError::Disconnected) => {
                debug!("** DC **");
                vc.push(String::from("~END~"));
                d = false;
            }
        }
    }

    if vc.len() > 0 {
        match sx.send(vc.join(" ")).await {
            Ok(_) => vc.clear(),
            Err(e) => error!("Error sending: {}", e.to_string()),
        }
    }


    info!("Stream finished");

    let result_buffer = result_buffer.clone();

    let text = crate::get_text!(result_buffer);
    let c_text = convert_text(text.as_str());
    result_buffer.set_text("");
    let mut end_iter = result_buffer.end_iter();
    result_buffer.insert_markup(&mut end_iter, c_text.as_str());
    drop(cc);
    info!("Ending chat");
    Ok(())
}

pub async fn ask_ollama(app_state: Arc<Context>, sx: Sender<String>) {
    let result_buffer = app_state.result_buffer().await;
    let text_buffer = app_state.text_buffer().await;
    let prompt = crate::get_text!(text_buffer);
    let model = app_state.conf.ollama_model.clone();
    let ollama = ollama_rs::Ollama::new(app_state.conf.ollama_url.as_str(), app_state.conf.ollama_port);

    let request = GenerationRequest::new(model, prompt.as_str());

    let mut stream = ollama.generate_stream(request).await.unwrap();
    let play = app_state.with_sound.lock().await;
    let mut vc = vec![];

    while let Some(res) = stream.next().await {
        match res {
            Ok(responses) => {
                let mut end_iter = result_buffer.end_iter();
                if *play {
                    for r in responses {
                        let content = &r.response;
                        let has_dot = content.contains(".") || content.contains("。");
                        result_buffer.insert(&mut end_iter, content);
                        vc.push(content.clone());
                        if has_dot || vc.len() > 10 {
                            match sx.send(vc.join(" ")).await {
                                Ok(_) => vc.clear(),
                                Err(e) => error!("Error sending: {}", e.to_string()),
                            }
                        }

                    }
                }
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    } 

    info!("Stream finished"); 

    if vc.len() > 0 {
        match sx.send(vc.join(" ")).await {
            Ok(_) => vc.clear(),
            Err(e) => error!("Error sending: {}", e.to_string()),
        }
    }


    let text = crate::get_text!(result_buffer);
    let c_text = convert_text(text.as_str());
    result_buffer.set_text("");
    let mut end_iter = result_buffer.end_iter();
    result_buffer.insert_markup(&mut end_iter, c_text.as_str());

    info!("Ending chat");

}
