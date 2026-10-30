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
        diarization: None,
        diarization_error: None,
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

#[test]
fn who_annotations_preserve_original_words_and_ambiguity() {
    let original = document().transcript.unwrap();
    let before = serde_json::to_string(&original).unwrap();
    let who = Diarization {
        model: "Who".into(),
        version: "v0.3.1".into(),
        raw_turns: None,
        known_speakers: None,
        turns: vec![
            SpeakerTurn {
                speaker: "who-0".into(),
                start: 0.,
                end: 0.55,
            },
            SpeakerTurn {
                speaker: "who-1".into(),
                start: 0.6,
                end: 1.3,
            },
        ],
    };
    who.validate(3.).unwrap();
    let annotated = who.annotate(&original);
    assert_eq!(annotated.segments.len(), 2);
    assert_eq!(
        annotated.segments[0].words[0].speaker.as_deref(),
        Some("who-0")
    );
    assert_eq!(
        annotated.segments[1].words[0].speaker.as_deref(),
        Some("who-1")
    );
    assert_eq!(annotated.text, original.text);
    assert_eq!(serde_json::to_string(&original).unwrap(), before);
    let overlap = Diarization {
        turns: vec![
            SpeakerTurn {
                speaker: "a".into(),
                start: 0.,
                end: 2.,
            },
            SpeakerTurn {
                speaker: "b".into(),
                start: 0.,
                end: 2.,
            },
        ],
        ..who
    };
    assert!(
        overlap
            .annotate(&original)
            .segments
            .iter()
            .all(|s| s.speaker.is_none())
    );
    assert_eq!(overlap.turns.len(), 2);
    let bad = Diarization {
        turns: vec![SpeakerTurn {
            speaker: "a".into(),
            start: f64::NAN,
            end: 1.,
        }],
        ..overlap
    };
    assert!(bad.validate(3.).is_err());
}
#[test]
fn legacy_documents_load_without_who_and_results_export_separately() {
    let mut d = document();
    let mut json = serde_json::to_value(&d).unwrap();
    json.as_object_mut().unwrap().remove("diarization");
    json.as_object_mut().unwrap().remove("diarization_error");
    assert!(
        serde_json::from_value::<TranscriptionDocument>(json)
            .unwrap()
            .diarization
            .is_none()
    );
    d.diarization = Some(Diarization {
        model: "Who".into(),
        version: "v0.3.1".into(),
        raw_turns: None,
        known_speakers: None,
        turns: vec![SpeakerTurn {
            speaker: "who-0".into(),
            start: 0.,
            end: 2.,
        }],
    });
    let export: serde_json::Value = serde_json::from_str(&d.export("json", true).unwrap()).unwrap();
    assert_eq!(export["diarization"]["turns"][0]["speaker"], "who-0");
    assert_eq!(export["transcript"]["segments"][0]["speaker"], "speaker_1");
    assert_eq!(d.export("txt", true).unwrap(), "Hej verden");
}

#[test]
fn known_count_uncertainty_and_legacy_diarization() {
    let old =
        r#"{"model":"Who","version":"old","turns":[{"speaker":"who-0","start":0.0,"end":1.0}]}"#;
    let mut d: Diarization = serde_json::from_str(old).unwrap();
    assert!(d.raw_turns.is_none() && d.known_speakers.is_none());
    d.raw_turns = Some(d.turns.clone());
    d.known_speakers = Some(KnownSpeakers {
        count: 1,
        method: "test".into(),
        cosine_threshold: 0.65,
    });
    d.turns.push(SpeakerTurn {
        speaker: "who-uncertain".into(),
        start: 0.5,
        end: 0.8,
    });
    let source = Transcript {
        text: "word".into(),
        timing: Timing::Word,
        timing_source: "test".into(),
        segments: vec![TranscriptSegment {
            id: "s".into(),
            text: "word".into(),
            start: Some(0.),
            end: Some(1.),
            speaker: None,
            words: vec![TimedWord {
                id: "w".into(),
                text: "word".into(),
                start: 0.,
                end: 1.,
                speaker: None,
            }],
        }],
    };
    assert_eq!(d.annotate(&source).segments[0].speaker, None);
    assert!(d.validate(2.).is_ok());
    d.known_speakers.as_mut().unwrap().count = 2;
    assert!(d.validate(2.).is_ok()); // Abstention may leave fewer IDs than the chosen count.
    d.known_speakers.as_mut().unwrap().count = 0;
    assert!(d.validate(2.).is_err());
}
