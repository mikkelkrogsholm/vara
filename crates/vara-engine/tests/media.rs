use vara_engine::{Cancellation, media};
fn wav(path: &std::path::Path, seconds: u32) {
    let mut w = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for i in 0..48000 * seconds {
        let s = ((i as f32 / 48000. * 440. * std::f32::consts::TAU).sin() * 8000.) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
}
#[test]
fn import_decodes_stereo_resamples_and_preserves_original() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("source.wav");
    let dst = dir.path().join("output.wav");
    wav(&src, 1);
    let before = std::fs::read(&src).unwrap();
    let duration = media::normalize(&src, &dst, &Cancellation::default()).unwrap();
    assert!((duration - 1.).abs() < 0.001);
    assert_eq!(std::fs::read(&src).unwrap(), before);
    let r = hound::WavReader::open(dst).unwrap();
    assert_eq!(r.spec().sample_rate, 16000);
    assert_eq!(r.spec().channels, 1);
    assert!(r.duration() >= 15999);
}
#[test]
fn unsupported_and_cancelled_imports_do_not_publish_work_audio() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("not-audio.mp4");
    let dst = dir.path().join("output.wav");
    std::fs::write(&src, b"no audio stream").unwrap();
    assert!(media::normalize(&src, &dst, &Cancellation::default()).is_err());
    assert!(!dst.exists());
    let c = Cancellation::default();
    c.cancel();
    assert!(media::normalize(&src, &dst, &c).is_err());
    assert!(!dst.exists());
    assert!(src.exists());
}
#[test]
fn chunk_processing_is_bounded_and_cancellation_prevents_the_next_request() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("long.wav");
    let mut w = hound::WavWriter::create(
        &src,
        hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for _ in 0..301 * 16000 {
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    let c = Cancellation::default();
    let mut calls = 0;
    let result = media::chunks(&src, &c, |path, offset, duration| {
        calls += 1;
        assert_eq!(offset, 0.);
        assert_eq!(duration, 300.);
        assert_eq!(
            hound::WavReader::open(path).unwrap().duration(),
            300 * 16000
        );
        c.cancel();
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
    assert!(src.exists());
}

#[test]
fn a_destination_alias_cannot_overwrite_the_original() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.wav");
    let alias = dir.path().join("alias.wav");
    wav(&source, 1);
    let before = std::fs::read(&source).unwrap();
    std::fs::hard_link(&source, &alias).unwrap();
    assert!(media::normalize(&source, &alias, &Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), before);
}

#[test]
fn published_playback_copies_remain_readable_when_the_original_moves() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.wav");
    let copy = dir.path().join("copy.wav");
    wav(&source, 1);
    media::normalize(&source, &copy, &Cancellation::default()).unwrap();
    std::fs::rename(&source, dir.path().join("moved.wav")).unwrap();
    assert!((media::normalized_duration(&copy).unwrap() - 1.).abs() < 0.001);
}
