use vara_core::*;
fn document() -> TranscriptionDocument {
    let engine = Settings::default()
        .engines
        .into_iter()
        .find(|e| e.provider == Provider::LocalParakeet)
        .unwrap();
    TranscriptionDocument {
        id: "doc1".into(),
        title: "Interview".into(),
        source_name: "interview.m4a".into(),
        source_path: "/original/interview.m4a".into(),
        created_at: 1,
        engine,
        language: Some("da".into()),
        stage: FileStage::Ready,
        error: None,
        duration: 3.,
        completed_seconds: 3.,
        waveform: vec![],
        transcript: Some(Transcript {
            text: "Hej verden".into(),
            timing: Timing::Word,
            timing_source: "asr".into(),
            segments: vec![TranscriptSegment {
                id: "s1".into(),
                text: "Hej verden".into(),
                start: Some(0.1),
                end: Some(1.2),
                speaker: Some("speaker_1".into()),
                words: vec![
                    TimedWord {
                        id: "w1".into(),
                        text: "Hej".into(),
                        start: 0.1,
                        end: 0.5,
                        speaker: Some("speaker_1".into()),
                    },
                    TimedWord {
                        id: "w2".into(),
                        text: "verden".into(),
                        start: 0.6,
                        end: 1.2,
                        speaker: Some("speaker_1".into()),
                    },
                ],
            }],
        }),
        edited_text: None,
        revision: 0,
        ai_text: None,
        ai_engine: None,
        speakers: Default::default(),
    }
}
#[test]
fn word_times_are_validated_without_rejecting_speaker_overlap() {
    let d = document();
    let mut t = d.transcript.unwrap();
    t.validate(3.).unwrap();
    t.segments[0].words[1].start = 0.2;
    t.validate(3.).unwrap();
    t.segments[0].words[0].end = f64::NAN;
    assert!(t.validate(3.).is_err());
}
#[test]
fn invalid_bounds_and_duplicate_word_ids_are_rejected() {
    let mut t = document().transcript.unwrap();
    t.segments[0].words[0].start = -1.;
    assert!(t.validate(3.).is_err());
    t.segments[0].words[0].start = 0.1;
    t.segments[0].words[1].id = "w1".into();
    assert!(t.validate(3.).is_err());
    t.segments[0].words[1].id = "w2".into();
    t.segments[0].end = Some(40.);
    assert!(t.validate(3.).is_err());
}
#[test]
fn edits_and_ai_never_inherit_original_timing() {
    let mut d = document();
    d.edited_text = Some("Rettet tekst".into());
    d.ai_text = Some("Resumé".into());
    assert_eq!(d.export("txt", false).unwrap(), "Rettet tekst");
    assert!(d.export("srt", false).is_err());
    assert!(
        d.export("vtt", true)
            .unwrap()
            .contains("00:00:00.100 --> 00:00:01.200")
    );
    assert!(d.export("srt", true).unwrap().contains("Hej verden"));
    let json = d.export("json", false).unwrap();
    assert!(json.contains("w1"));
    assert!(json.contains("Resumé"));
    assert_eq!(d.transcript.unwrap().text, "Hej verden");
}
#[test]
fn text_only_models_cannot_export_invented_subtitles() {
    let mut d = document();
    d.transcript = Some(Transcript::plain("Ord uden tider".into()));
    d.transcript.as_ref().unwrap().validate(3.).unwrap();
    assert!(d.export("vtt", true).is_err());
    assert_eq!(d.export("txt", true).unwrap(), "Ord uden tider");
}
#[test]
fn documents_survive_restart_without_entering_dictation_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("transcriptions.sqlite3");
    let mut d = document();
    d.edited_text = Some("Min rettelse".into());
    d.revision = 2;
    {
        let s = TranscriptionStore::open(&path).unwrap();
        s.save(&d).unwrap();
    }
    let s = TranscriptionStore::open(&path).unwrap();
    let all = s.all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].revision, 2);
    assert_eq!(all[0].text(), "Min rettelse");
    assert_eq!(all[0].transcript.as_ref().unwrap().text, "Hej verden");
    let history = Store::open(&dir.path().join("vara.sqlite3")).unwrap();
    assert!(history.history(None, None).unwrap().is_empty());
    s.delete(&d.id).unwrap();
    assert!(s.all().unwrap().is_empty());
}

#[test]
fn importing_a_batch_is_atomic_on_storage_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = TranscriptionStore::open(&dir.path().join("files.db")).unwrap();
    let first = document();
    let mut second = first.clone();
    second.id = "second".into();
    store.save(&first).unwrap();
    assert!(store.save_batch(&[second, first]).is_err());
    assert_eq!(store.all().unwrap().len(), 1);
}
