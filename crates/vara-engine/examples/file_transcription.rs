//! Developer verification: private fixture input, no microphone, keys or OS insertion.
use std::{collections::BTreeMap, path::PathBuf};
use vara_core::*;
use vara_engine::{Cancellation, Engines, media};
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: file_transcription MODELS INPUT OUTPUT_JSON".into());
    }
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let dir = tempfile::tempdir()?;
    let audio = dir.path().join("normalized.wav");
    let cancel = Cancellation::default();
    let duration = media::normalize(std::path::Path::new(&args[1]), &audio, &cancel)?;
    let engines = Engines::new(PathBuf::from(&args[0]))?;
    let engine = Settings::default()
        .engines
        .into_iter()
        .find(|e| e.provider == Provider::LocalParakeet)
        .unwrap();
    let started = std::time::Instant::now();
    let transcript = engines.transcribe_file(&engine, &audio, None, None, &cancel, &|_| {})?;
    transcript.validate(duration)?;
    let words = transcript
        .segments
        .iter()
        .map(|s| s.words.len())
        .sum::<usize>();
    let doc = TranscriptionDocument {
        id: "fixture-danish".into(),
        title: "Dansk prøve".into(),
        source_name: std::path::Path::new(&args[1])
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        source_path: args[1].clone(),
        created_at: 1792953000,
        engine,
        language: None,
        stage: FileStage::Ready,
        error: None,
        duration,
        completed_seconds: duration,
        waveform: media::waveform(&audio, &cancel)?,
        transcript: Some(transcript),
        edited_text: None,
        revision: 0,
        ai_text: None,
        ai_engine: None,
        speakers: BTreeMap::new(),
    };
    std::fs::write(&args[2], doc.export("json", true)?)?;
    let subtitles = doc.export("srt", true)?;
    assert!(!subtitles.is_empty());
    println!(
        "duration={duration:.3}s words={words} elapsed={:.3}s timing={:?}",
        started.elapsed().as_secs_f64(),
        doc.transcript.as_ref().unwrap().timing
    );
    println!("{}", doc.text());
    Ok(())
}
