use pv_recorder::{PvRecorderBuilder, PvRecorder};
use tokio::sync::Mutex;
use gtk::TextBuffer;
use gtk::prelude::*;
use crate::Language;
use std::thread::JoinHandle;
use crate::config::Config;
use anyhow::{Result, anyhow};
use tracing::{info, debug, error};
use std::env::current_exe;

const CONF: &str = "app.toml";

pub struct UiContext {
    text_buffer: TextBuffer,
    pub result_buffer: TextBuffer,
}

pub struct RecContext {
    recorder: PvRecorder,
    pub audio_data: Vec<i16>,
    pub rec_c: bool,
    pub handle: Option<std::thread::JoinHandle<()>>,
}

pub struct Context {
    pub ui: Mutex<UiContext>,
    pub re: Mutex<RecContext>,
    pub language: Mutex<Option<Language>>,
    pub conf: Config,
    pub ai_chat: Mutex<Option<crate::AiChat>>,
    pub with_sound: Mutex<bool>,
}

unsafe impl Send for Context {}
unsafe impl Send for UiContext {}

impl UiContext {
    pub fn new(tv: &TextBuffer, rv: &TextBuffer) -> Self {
        Self { text_buffer: tv.clone(), result_buffer: rv.clone() }
    }

    pub fn append_text(&mut self, s: &str) {
        let mut end_iter = self.text_buffer.end_iter();
        self.text_buffer.insert(&mut end_iter, s);
    }

    pub fn append_result(&mut self, s: &str) {
        let mut end_iter = self.result_buffer.end_iter();
        self.result_buffer.insert(&mut end_iter, s);
    }

    pub fn clear_text(&mut self) {
        let mut start = self.text_buffer.start_iter();
        let mut end = self.text_buffer.end_iter();
        self.text_buffer.delete(&mut start, &mut end);
    }

    pub fn clear_result(&mut self) {
        let mut start = self.result_buffer.start_iter();
        let mut end = self.result_buffer.end_iter();
        self.result_buffer.delete(&mut start, &mut end);
    }

}

impl RecContext {
    pub fn new() -> Self {
        Self {
            recorder: PvRecorderBuilder::new(512).device_index(0).init().expect("Recorder init error"),
            audio_data: vec![],
            rec_c: false,
            handle: None,
        }
    }

    pub fn reset_buffer(&mut self) {
        self.audio_data = vec![];
    }

    pub fn set_rec_device(&mut self, di: i32) -> Result<()> {
        self.recorder.stop()?;
        self.recorder = PvRecorderBuilder::new(512).device_index(di).init().expect("Recorder init error");
        self.recorder.start()?;
        Ok(())
    }

    pub fn buffer_len(&self) -> usize {
        self.audio_data.len()
    }

    pub fn get_ad(&self) -> &[i16] {
        self.audio_data.as_slice()
    }

    pub fn toggle(&mut self) -> bool {
        let v = !self.rec_c;
        self.rec_c = v;
        debug!("Toggle: {}", v);
        if v {
            let _ = self.recorder.start();
        } else {
            let _ = self.recorder.stop();
        }
        self.rec_c
    }

    pub fn rec(&mut self, v: bool) -> Result<()> {
        self.rec_c = v;
        if v {
            self.recorder.start()?;
        }  else {
            self.recorder.stop()?;
        }
        Ok(())
    }

    pub fn read(&mut self) -> Result<()> {
        let r = self.recorder.read();
        match r {
            Ok(z) => {
                self.audio_data.extend_from_slice(&z);
                debug!("AU len {}", self.audio_data.len());
                return Ok(())
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn clear(&mut self) {
        self.audio_data.clear();
    }
}

impl Context {
    pub fn new(tv: &TextBuffer, rv: &TextBuffer) -> Self {
        info!("Initializing Context");
        let exe =  current_exe().unwrap();
        let ce = exe.parent().unwrap();
        let config_path = ce.join(CONF);

        Self {
            ui: Mutex::new(UiContext::new(tv,rv)),
            re: Mutex::new(RecContext::new()),
            language: Mutex::new(Some(Language::EN)),
            conf: toml::from_str(
                std::fs::read_to_string( 
                    config_path.to_str().unwrap_or("./app.toml") 
                    ).unwrap_or(std::fs::read_to_string("./app.toml").unwrap()).as_str() 
                ).unwrap(),
            ai_chat: Mutex::new(Some(crate::AiChat::ChatGPT)),
            with_sound: Mutex::new(false),
        }
    }

    pub async fn set_rec_device(&self, di: i32) -> Result<()> {
        debug!("Setting record device: {}", di);
        let mut h = self.re.lock().await;
        h.set_rec_device(di)?;
        Ok(())
    }

    pub async fn join_handle(&self) -> Result<(), Box<dyn std::any::Any + Send>> {
        let mut h = self.re.lock().await;
        if let Some(a) = h.handle.take() {
            a.join()?;
        }
        debug!("Thread stopped");
        Ok(())
    }

    pub async fn set_handle(&self, handle: JoinHandle<()>) {
        let mut re = self.re.lock().await;
        re.handle = Some(handle);
    }

    pub async fn text_buffer(&self) -> TextBuffer {
        self.ui.lock().await.text_buffer.clone()
    }

    pub async fn result_buffer(&self) -> TextBuffer {
        self.ui.lock().await.result_buffer.clone()
    }

    pub async fn au_buffer_len(&self) -> usize {
        self.re.lock().await.buffer_len()
    }

    pub async fn toggle_rec(&self) -> bool {
        debug!("Toggle outer");
        let mut re = self.re.lock().await;
        debug!("Toggle outer got lock");
        let result = re.toggle();
        debug!("Toggle outer result: {}", result);
        result
    }

    pub async fn dispose(&self) -> Result<()> {
        let mut rlock = self.re.lock().await;
        crate::report_err!(rlock.recorder.stop());
        
        if let Some(h) = rlock.handle.take() {
            let _ = h.join();
        }

        drop(rlock);
        
        debug!("Disposed");
        Ok(())
    }

    pub async fn clear_audio(&self) {
        let mut r = self.re.lock().await;
        r.clear();
    }
}
