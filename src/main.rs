#![allow(dead_code)]
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, ScrolledWindow, TextView, Box, DropDown, Label, CheckButton};
use std::sync::Arc;
use crate::context::Context;
use tracing::{debug, error, info};

mod context;
mod helper;
mod chat;
mod config;
mod transcribe;

make_enum!(AiChat, [ChatGPT, Grok, Deepseek, Ollama]);
make_enum!(Language, [EN,PL,CN,DE,FR,ES,RU,TR,JP]);

use elevenlabs_rs::{ElevenLabsClient, Model};
use elevenlabs_rs::endpoints::genai::tts::{TextToSpeech, TextToSpeechBody};
use elevenlabs_rs::utils::play;

const APP_NAME: &str = "gChatter 0.3.0";
const APP_ID: &str = "org.gnome.gChatter.Devel";

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_height(550)
        .default_width(700)
        .build();

    let text_view = TextView::builder()
        .editable(true)
        .cursor_visible(true)
        .accepts_tab(true)
        .height_request(160)
        .wrap_mode(gtk::WrapMode::Word)
        .build();


    let result_view = TextView::builder()
        .editable(false)
        .cursor_visible(true)
        .margin_top(10)
        .wrap_mode(gtk::WrapMode::Word)
        .build();

    let ctx = Arc::new(Context::new(&text_view.buffer(), &result_view.buffer()));
    debug!("Context ready");

    let (chat_sx,chat_rx) = async_channel::unbounded::<String>();
    let st = ctx.clone();
    let elh = tokio::spawn(async move {
    let st = st.clone();
        if let Some(c) = &st.conf.eleven {
            let key = &c.key;
            //let url = &c.url;
            let voice = &c.model;
            let client = ElevenLabsClient::new(key.as_str());
            info!("Found elevenlabs config");
            while let Ok(txt) = chat_rx.recv().await {
                if !txt.chars().any(|c| matches!(c, 'a'..='z')) {
                    // Skip if nothing to read.
                    continue;
                }
                debug!("read: {}", txt);
                let body = TextToSpeechBody::new(txt)
                    .with_model_id(Model::ElevenMultilingualV2);
                let endpoint = TextToSpeech::new(voice, body);
                match client.hit(endpoint).await {
                    Ok(speech) => {
                        debug!("playing");
                        crate::report_err!(play(speech));
                    }
                    Err(e) => {
                        error!("tts error: {}", e.to_string());
                    }
                }
            }
        }
    });

    let ai_sel = enum_dd!(AiChat, 0, 100);
    let st = ctx.clone();
    ai_sel.connect_selected_item_notify(move |r| {
        let sel = r.selected();
        let st = st.clone();
        let item = AiChat::ALL[sel as usize];
        glib::spawn_future_local(async move {
            let mut a = st.ai_chat.lock().await;
            *a = Some(item);
        });
    });

    let st = ctx.clone();
    let language_sel = enum_dd!(Language, 5);
    language_sel.connect_selected_item_notify(move |r| {
        let sel = r.selected();
        let st = st.clone();
        glib::spawn_future_local(async move {
            let mut a = st.language.lock().await;
            *a = Some(Language::ALL[sel as usize]);
        });
    });
    
    let s_result_view = ScrolledWindow::builder()
        .child(&result_view)
        .min_content_height(310)
        .build();

    let idc_ask = Button::builder()
        .label("Ask")
        .build();
    let devices = helper::device_dd();
    let st = ctx.clone();
    devices.connect_selected_item_notify(move |r| {
        let st = st.clone();
        let isel = r.selected() as i32;
        debug!("Selected: {}", isel);
        glib::spawn_future_local(async move {
            match st.set_rec_device(isel).await {
                Ok(_) => debug!("Device set"),
                Err(e) => error!("Error setting device: {}", e.to_string()),
            }
        });
    });

    let idc_rec = Button::builder()
        .label("Rec")
        .margin_start(5)
        .build();

    let idc_play = CheckButton::builder()
        .label("Play")
        .build();

    let st = ctx.clone();
    idc_play.connect_toggled(move |b| {
        let v = b.is_active();
        let st = st.clone();
        glib::spawn_future_local(async move {
            let mut p = st.with_sound.lock().await;
            *p = v;
            debug!("Set play to {}", v);
        });
        
    });

    let (sx,rx) = async_channel::unbounded::<bool>();
    let s = sx.clone();
    let st = ctx.clone();
    idc_rec.connect_clicked(move |_| {
        let s = s.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            st,
            async move {
                let v = !st.toggle_rec().await;
                debug!("rec: {}", v);
                if v {
                    debug!("Trying to stop");
                    s.send(true).await.expect("Failed to send false");
                    match st.join_handle().await {
                        Ok(_) => debug!("Joined handle"),
                        Err(e) => error!("Error joining handle: {:?}", e),
                    }
                } else {
                    s.send(false).await.expect("Failed to send false");
                    debug!("Trying to start");
                    let st = st.clone();
                    st.clear_audio().await;
                    let st2 = st.clone();
                    let h = std::thread::spawn(move || {
                        debug!("In da thread");
                        while st.re.blocking_lock().rec_c {
                            if let Err(e) = st.re.blocking_lock().read() {
                                error!("Error reading: {}", e.to_string());
                            }
                        }
                    });
                    st2.set_handle(h).await;
                }
            }
        ));
    });

    let idc_tr = Button::builder()
        .label("Transcribe")
        .margin_start(5)
        .build();

    let status_label = Label::builder()
        .label("stopped")
        .margin_start(10)
        .build();
    let r = rx.clone();
    glib::spawn_future_local(glib::clone!(
        #[weak]
        devices,
        #[weak]
        idc_tr,
        #[weak]
        status_label,
        async move {
            while let Ok(en) = r.recv().await {
                devices.set_sensitive(en);
                idc_tr.set_sensitive(en);
                status_label.set_text(if en { "" } else { "recording" });
            }
        }
    ));

    let st = ctx.clone();
    let st2 = ctx.clone();
    let s = sx.clone();
    idc_tr.connect_clicked(move |_| {
        let st = st.clone();
        let st2 = st2.clone();
        let s = s.clone();

        glib::spawn_future_local(async move {
            s.send(false).await.expect("Failed to send from TR");
            let res = transcribe::au_to_text(st).await;
            match res {
                Ok(r) => {
                    let a = st2.text_buffer().await;
                    a.set_text(r.as_str());
                },
                Err(e) => {
                    error!("Error transcribing: {}", e.to_string());
                },
            }
            s.send(true).await.expect("Failed to send from TR");
        });
    });

    let ids_dev = Label::builder()
        .label("device")
        .margin_start(5)
        .build();
    idc_ask.set_sensitive(false);
    //let ask_rc = std::rc::Rc
    connect_text_buffer_to_button(&text_view, &idc_ask);

    let idc_clearq = Button::builder()
        .label("CQ")
        .width_request(60)
        .margin_start(5)
        .build();

    let st = ctx.clone();
    idc_clearq.connect_clicked(move |_| {
        match st.ui.try_lock() {
            Ok(mut r) => {
                r.clear_text();
            }
            Err(r) => {
                debug!("Error acquiring lock: {}", r.to_string());
            }
        }
    });


    let hbox = row!(5,[ai_sel, ids_dev, devices, status_label]);
    let bhbox = row!(5,[idc_ask, idc_rec, language_sel, idc_tr, idc_clearq, idc_play]);
    let vbox = column![text_view, s_result_view, hbox, bhbox];

    let st = ctx.clone();
    let st2 = ctx.clone();

    idc_ask.connect_clicked(move |_| {
        let st = st.clone();
        let st2 = st2.clone();
        let chat_sx = chat_sx.clone();
        glib::spawn_future_local(async move {
            let ai = st.ai_chat.lock().await.clone();
            let result_buffer = st.result_buffer().await;
            clear_text!(result_buffer);
            match ai {
                Some(AiChat::Ollama) => {
                    glib::spawn_future_local(async move {
                        let chat_sx = chat_sx.clone();
                        chat::ask_ollama(st2, chat_sx).await;
                    });
                }
                Some(AiChat::Grok) | Some(AiChat::ChatGPT) | Some(AiChat::Deepseek) => {
                    glib::spawn_future_local(async move {
                        let chat_sx = chat_sx.clone();
                        if let Err(e) = chat::ask_chat(st2, chat_sx).await {
                            error!("Error asking online chat: {}", e.to_string());
                        }
                    });
                }
                None => {
                    error!("No chat selected");
                }
            }
        });
    });

       
    window.set_child(Some(&vbox));
    window.present();
    window.connect_close_request(move |_| {
        let st = ctx.clone();
        elh.abort();
        glib::spawn_future_local(async move {
            st.dispose().await.expect("Disposing failed, you may need to shut me down by force");
        });
        glib::Propagation::Proceed
    });

}

#[tokio::main]
async fn main() -> glib::ExitCode {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn connect_text_buffer_to_button<'a>(text_view: &TextView, button: &'a Button) {
    let buffer = text_view.buffer();

    // Initial check (in case TextView starts with content)
    update_button_state(&buffer, &button);

    // Connect the "changed" signal to update button state dynamically
    buffer.connect_changed(glib::clone!(
            #[weak]
            button,
            #[weak]
            buffer,
            move |_| {
                update_button_state(&buffer, &button);
            }));
}

fn update_button_state<'a>(buffer: &gtk::TextBuffer, button: &'a Button) {
    let (start, end) = buffer.bounds();
    let is_empty = buffer.text(&start, &end, false).is_empty();

    // Enable button only if TextView is NOT empty
    button.set_sensitive(!is_empty);
}
