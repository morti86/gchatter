use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::sync::Arc;
use crate::context::Context;
use tracing::debug;

pub async fn au_to_text(st: Arc<Context>) -> anyhow::Result<String> {
    let a = st.clone();
    a.ui.lock().await.clear_text();
    let path_to_model = a.conf.whisper_model.as_str();
    let lang = a.language.lock().await.unwrap_or(crate::Language::EN);
    let ctx = WhisperContext::new_with_params(
        path_to_model,
	WhisperContextParameters::default())?;
    let mut res = String::new();

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    let lang = lang.to_string().to_lowercase();
    params.set_language(Some(lang.as_str()));
    let mut state = ctx.create_state()?;

    let au = a.re.lock().await;
    let blen = au.buffer_len();
    let mut inter_samples = vec![Default::default(); blen];
    whisper_rs::convert_integer_to_float_audio(au.get_ad(), &mut inter_samples)?;
    let _r = state.full(params, &inter_samples[..])?;
    let num_segments = state.full_n_segments()?;
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)?;
        debug!("- {}", segment);
        res.push_str(segment.as_str());
    }

    Ok(res)
}

