//! File-transcription documents. Independent of dictation sessions and OS delivery.
use crate::{EngineConfig, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Timing {
    None,
    Segment,
    Word,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedWord {
    pub id: String,
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub speaker: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: String,
    pub text: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub speaker: Option<String>,
    pub words: Vec<TimedWord>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub segments: Vec<TranscriptSegment>,
    pub timing: Timing,
    pub timing_source: String,
}
/// Independent, model-produced speaker intervals. Overlap is preserved, not forced
/// into an invented single speaker. Original ASR text/times are never modified.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakerTurn {
    pub speaker: String,
    pub start: f64,
    pub end: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diarization {
    pub model: String,
    pub version: String,
    pub turns: Vec<SpeakerTurn>,
}
impl Diarization {
    pub fn validate(&self, duration: f64) -> Result<()> {
        if !duration.is_finite()
            || duration <= 0.
            || self.turns.iter().any(|t| {
                t.speaker.is_empty()
                    || !t.start.is_finite()
                    || !t.end.is_finite()
                    || t.start < 0.
                    || t.end <= t.start
                    || t.end > duration + 0.1
            })
        {
            return Err(Error::Invalid(
                "Taleropdelingen har ugyldige tidsdata.".into(),
            ));
        }
        Ok(())
    }
    /// Label only intervals with one clear speaker. Silence and simultaneous
    /// speakers remain unassigned; the full overlapping turns remain in JSON.
    pub fn annotate(&self, source: &Transcript) -> Transcript {
        let label = |start: f64, end: f64| -> Option<String> {
            let active: std::collections::BTreeSet<_> = self
                .turns
                .iter()
                .filter(|t| t.start < end && t.end > start)
                .map(|t| t.speaker.as_str())
                .collect();
            if active.len() != 1 {
                return None;
            }
            let speaker = *active.first()?;
            let coverage: f64 = self
                .turns
                .iter()
                .filter(|t| t.speaker == speaker)
                .map(|t| (t.end.min(end) - t.start.max(start)).max(0.))
                .sum();
            (coverage >= (end - start) * 0.5).then(|| speaker.to_owned())
        };
        let mut out = source.clone();
        out.segments.clear();
        for segment in &source.segments {
            if segment.words.is_empty() {
                let mut s = segment.clone();
                s.speaker = segment
                    .start
                    .zip(segment.end)
                    .and_then(|(a, b)| label(a, b));
                out.segments.push(s);
                continue;
            }
            let first_group = out.segments.len();
            for word in &segment.words {
                let mut w = word.clone();
                w.speaker = label(w.start, w.end);
                if out.segments.len() == first_group
                    || out.segments.last().is_none_or(|s| s.speaker != w.speaker)
                {
                    out.segments.push(TranscriptSegment {
                        id: format!("{}-who-{}", segment.id, out.segments.len()),
                        text: String::new(),
                        start: Some(w.start),
                        end: Some(w.end),
                        speaker: w.speaker.clone(),
                        words: vec![],
                    });
                }
                let s = out.segments.last_mut().unwrap();
                if !s.text.is_empty() {
                    s.text.push(' ');
                }
                s.text.push_str(&w.text);
                s.end = Some(w.end);
                s.words.push(w);
            }
        }
        out
    }
}
impl Transcript {
    pub fn plain(text: String) -> Self {
        Self {
            segments: vec![TranscriptSegment {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.clone(),
                start: None,
                end: None,
                speaker: None,
                words: vec![],
            }],
            text,
            timing: Timing::None,
            timing_source: "none".into(),
        }
    }
    pub fn validate(&self, duration: f64) -> Result<()> {
        let bad = || Error::Invalid("Transskriptionen har ugyldige tidsdata.".into());
        if !duration.is_finite() || duration <= 0. {
            return Err(bad());
        }
        let interval = |a: f64, b: f64| {
            a.is_finite() && b.is_finite() && a >= 0. && b >= a && b <= duration + 0.1
        };
        let mut ids = std::collections::HashSet::new();
        for s in &self.segments {
            if !ids.insert(&s.id) {
                return Err(bad());
            }
            match (s.start, s.end) {
                (Some(a), Some(b)) if interval(a, b) => (),
                (None, None) if self.timing == Timing::None => (),
                _ => return Err(bad()),
            }
            for w in &s.words {
                if !ids.insert(&w.id) || !interval(w.start, w.end) || self.timing != Timing::Word {
                    return Err(bad());
                }
            }
        }
        if self.timing == Timing::Word && !self.segments.iter().any(|s| !s.words.is_empty()) {
            return Err(bad());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileStage {
    Queued,
    Decoding,
    Transcribing,
    Ready,
    Failed,
    Cancelled,
}
impl FileStage {
    pub fn busy(self) -> bool {
        matches!(self, Self::Queued | Self::Decoding | Self::Transcribing)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionDocument {
    pub id: String,
    pub title: String,
    pub source_name: String,
    pub source_path: String,
    pub created_at: i64,
    pub engine: EngineConfig,
    pub language: Option<String>,
    pub stage: FileStage,
    pub error: Option<String>,
    pub duration: f64,
    pub completed_seconds: f64,
    #[serde(default)]
    pub waveform: Vec<f32>,
    pub transcript: Option<Transcript>,
    pub edited_text: Option<String>,
    pub revision: u64,
    pub ai_text: Option<String>,
    pub ai_engine: Option<EngineConfig>,
    pub speakers: BTreeMap<String, String>,
    #[serde(default)]
    pub diarization: Option<Diarization>,
    #[serde(default)]
    pub diarization_error: Option<String>,
}
impl TranscriptionDocument {
    pub fn text(&self) -> &str {
        self.edited_text.as_deref().unwrap_or_else(|| {
            self.transcript
                .as_ref()
                .map(|t| t.text.as_str())
                .unwrap_or("")
        })
    }
    pub fn export(&self, format: &str, original: bool) -> Result<String> {
        let t = self
            .transcript
            .as_ref()
            .ok_or_else(|| Error::Invalid("Transskriptionen er ikke klar.".into()))?;
        let text = if original {
            t.text.as_str()
        } else {
            self.text()
        };
        match format {
            "txt" => Ok(text.into()),
            "md" => Ok(format!("# {}\n\n{}\n", self.title, text)),
            "json" => Ok(serde_json::to_string_pretty(self)?),
            "srt" | "vtt" => {
                if (!original && self.edited_text.is_some()) || t.timing == Timing::None {
                    return Err(Error::Invalid("Undertekster kræver tidsjusteret tekst. Vælg den tidskodede original, eller eksportér som tekst.".into()));
                }
                t.validate(self.duration)?;
                let mut cues: Vec<(f64, f64, String)> = vec![];
                for s in &t.segments {
                    if s.words.is_empty() {
                        if let (Some(a), Some(b)) = (s.start, s.end) {
                            cues.push((a, b, s.text.clone()));
                        }
                    } else {
                        let mut group: Vec<&TimedWord> = vec![];
                        for w in &s.words {
                            if !group.is_empty()
                                && (w.end - group[0].start > 5.
                                    || group
                                        .iter()
                                        .map(|v| v.text.chars().count() + 1)
                                        .sum::<usize>()
                                        + w.text.chars().count()
                                        > 72)
                            {
                                cues.push((
                                    group[0].start,
                                    group.last().unwrap().end,
                                    group
                                        .iter()
                                        .map(|v| v.text.as_str())
                                        .collect::<Vec<_>>()
                                        .join(" "),
                                ));
                                group.clear();
                            }
                            group.push(w);
                        }
                        if !group.is_empty() {
                            cues.push((
                                group[0].start,
                                group.last().unwrap().end,
                                group
                                    .iter()
                                    .map(|v| v.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            ));
                        }
                    }
                }
                let stamp = |seconds: f64| {
                    let ms = (seconds * 1000.).round() as u64;
                    format!(
                        "{:02}:{:02}:{:02}{}{:03}",
                        ms / 3600000,
                        ms / 60000 % 60,
                        ms / 1000 % 60,
                        if format == "srt" { "," } else { "." },
                        ms % 1000
                    )
                };
                let mut out = if format == "vtt" {
                    "WEBVTT\n\n".into()
                } else {
                    String::new()
                };
                for (i, (a, b, text)) in cues.iter().enumerate() {
                    out += &format!(
                        "{}\n{} --> {}\n{}\n\n",
                        i + 1,
                        stamp(*a),
                        stamp(*b),
                        text.replace("-->", "→")
                    );
                }
                Ok(out)
            }
            _ => Err(Error::Invalid("Ukendt eksportformat.".into())),
        }
    }
}

/// Dedicated database keeps file jobs separate from dictation retention and recovery.
pub struct TranscriptionStore {
    db: rusqlite::Connection,
}
impl TranscriptionStore {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let db = rusqlite::Connection::open(path).map_err(Error::FileStorage)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA secure_delete=ON; CREATE TABLE IF NOT EXISTS documents(id TEXT PRIMARY KEY,created_at INTEGER NOT NULL,value TEXT NOT NULL);").map_err(Error::FileStorage)?;
        Ok(Self { db })
    }
    pub fn all(&self) -> Result<Vec<TranscriptionDocument>> {
        let mut stmt = self
            .db
            .prepare("SELECT value FROM documents ORDER BY created_at DESC,rowid DESC")
            .map_err(Error::FileStorage)?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(Error::FileStorage)?;
        rows.map(|r| Ok(serde_json::from_str(&r.map_err(Error::FileStorage)?)?))
            .collect()
    }
    pub fn save(&self, d: &TranscriptionDocument) -> Result<()> {
        self.db.execute("INSERT INTO documents(id,created_at,value) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET value=excluded.value",rusqlite::params![d.id,d.created_at,serde_json::to_string(d)?]).map_err(Error::FileStorage)?;
        Ok(())
    }
    pub fn save_batch(&mut self, documents: &[TranscriptionDocument]) -> Result<()> {
        let tx = self.db.transaction().map_err(Error::FileStorage)?;
        for d in documents {
            tx.execute(
                "INSERT INTO documents(id,created_at,value) VALUES(?1,?2,?3)",
                rusqlite::params![d.id, d.created_at, serde_json::to_string(d)?],
            )
            .map_err(Error::FileStorage)?;
        }
        tx.commit().map_err(Error::FileStorage)
    }
    pub fn delete(&self, id: &str) -> Result<()> {
        self.db
            .execute("DELETE FROM documents WHERE id=?1", [id])
            .map_err(Error::FileStorage)?;
        Ok(())
    }
}
