//! Durable file jobs. Never focus, paste, or use dictation recovery/audio retention.
use parking_lot::Mutex;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tauri::Emitter;
use vara_core::*;
use vara_engine::{Cancellation, Engines};
type Result<T> = std::result::Result<T, String>;
struct Inner {
    store: TranscriptionStore,
    documents: Vec<TranscriptionDocument>,
    cancel: BTreeMap<String, Cancellation>,
    worker: bool,
}
pub struct Files {
    inner: Mutex<Inner>,
    root: PathBuf,
    engines: Engines,
    app: tauri::AppHandle,
}
impl Files {
    pub fn new(app: tauri::AppHandle, root: PathBuf) -> Result<Arc<Self>> {
        let media = root.join("transcriptions");
        std::fs::create_dir_all(&media)
            .map_err(|_| "Transskriptionsmappen kunne ikke oprettes.")?;
        let store = TranscriptionStore::open(&root.join("transcriptions.sqlite3"))
            .map_err(|e| e.to_string())?;
        let mut documents = store.all().map_err(|e| e.to_string())?;
        for d in &mut documents {
            if d.stage.busy() {
                d.stage = FileStage::Failed;
                d.error = Some("Vara blev lukket under behandlingen. Genprøv filen.".into());
                store.save(d).map_err(|e| e.to_string())?;
            }
        }
        // Partial normalized files and cancelled/deleted job files are never replayed.
        for p in std::fs::read_dir(&media)
            .map_err(|_| "Lydmappen kunne ikke læses.")?
            .flatten()
        {
            let path = p.path();
            if path.extension().is_some_and(|e| e == "part")
                || !documents
                    .iter()
                    .any(|d| path == media.join(format!("{}.wav", d.id)))
            {
                let _ = std::fs::remove_file(path);
            }
        }
        let engines = Engines::new(root.join("models"))?;
        Ok(Arc::new(Self {
            inner: Mutex::new(Inner {
                store,
                documents,
                cancel: BTreeMap::new(),
                worker: false,
            }),
            root: media,
            engines,
            app,
        }))
    }
    fn emit(&self) {
        let _ = self.app.emit("vara:files", ());
    }
    fn path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.wav"))
    }
    fn save(i: &mut Inner, d: TranscriptionDocument) -> Result<()> {
        i.store.save(&d).map_err(|e| e.to_string())?;
        if let Some(old) = i.documents.iter_mut().find(|v| v.id == d.id) {
            *old = d;
        } else {
            i.documents.insert(0, d);
        }
        Ok(())
    }
    fn update(&self, id: &str, work: impl FnOnce(&mut TranscriptionDocument)) -> Result<()> {
        let mut i = self.inner.lock();
        let mut d = i
            .documents
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or("Transskriptionen findes ikke længere.")?;
        work(&mut d);
        Self::save(&mut i, d)
    }
    fn update_active(
        &self,
        id: &str,
        c: &Cancellation,
        work: impl FnOnce(&mut TranscriptionDocument),
    ) -> Result<()> {
        let mut i = self.inner.lock();
        c.check()?;
        let mut d = i
            .documents
            .iter()
            .find(|d| d.id == id && d.stage.busy())
            .cloned()
            .ok_or("Filjobbet er annulleret eller slettet.")?;
        work(&mut d);
        Self::save(&mut i, d)
    }
    pub fn stop(&self) {
        let i = self.inner.lock();
        for c in i.cancel.values() {
            c.cancel();
        }
    }
    pub fn command(self: &Arc<Self>, action: &str, p: Value, settings: &Settings) -> Result<Value> {
        let id = || document_id(&p);
        match action {
            "file_list" => {
                let i = self.inner.lock();
                return Ok(
                    json!({"documents":i.documents,"media":i.documents.iter().filter(|d|self.path(&d.id).is_file()).map(|d|(d.id.clone(),self.path(&d.id).to_string_lossy().to_string())).collect::<BTreeMap<_,_>>()}),
                );
            }
            "file_pick" => {
                let files = rfd::FileDialog::new()
                    .set_title("Vælg lyd eller video")
                    .pick_files()
                    .unwrap_or_default();
                let list=files.into_iter().filter(|p|p.is_file()).map(|p|json!({"path":p.to_string_lossy(),"name":p.file_name().unwrap_or_default().to_string_lossy(),"bytes":p.metadata().map(|m|m.len()).unwrap_or(0)})).collect::<Vec<_>>();
                return Ok(json!(list));
            }
            "file_add" => {
                let paths: Vec<String> =
                    serde_json::from_value(p["paths"].clone()).map_err(|_| "Vælg filer først.")?;
                if paths.is_empty() || paths.len() > 100 {
                    return Err("Vælg mellem 1 og 100 filer ad gangen.".into());
                }
                let engine = settings
                    .engines
                    .iter()
                    .find(|e| {
                        Some(e.id.as_str()) == p["engine"].as_str() && e.kind == EngineKind::Speech
                    })
                    .cloned()
                    .ok_or("Vælg en talemodel.")?;
                if settings.strict_local && engine.locality != Locality::OnDevice {
                    return Err("Kun lokal behandling er slået til i Privatliv.".into());
                }
                if engine.locality == Locality::OnDevice && !self.engines.ready(&engine.id) {
                    return Err("Hent talemodellen under Modeller og nøgler først.".into());
                }
                // Validate the whole batch before publishing any queued jobs.
                for path in &paths {
                    if !std::fs::metadata(path).is_ok_and(|m| m.is_file()) {
                        return Err(
                            "En kildefil mangler eller er ikke en almindelig fil. Vælg den igen."
                                .into(),
                        );
                    }
                }
                let language = p["language"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned);
                let mut i = self.inner.lock();
                let mut ids = vec![];
                let mut batch = vec![];
                for path in paths {
                    let source = PathBuf::from(&path);
                    let name = source
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let d = TranscriptionDocument {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: source
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        source_name: name,
                        source_path: path,
                        created_at: crate::service::now(),
                        engine: engine.clone(),
                        language: language.clone(),
                        stage: FileStage::Queued,
                        error: None,
                        duration: 0.,
                        completed_seconds: 0.,
                        waveform: vec![],
                        transcript: None,
                        edited_text: None,
                        revision: 0,
                        ai_text: None,
                        ai_engine: None,
                        speakers: BTreeMap::new(),
                    };
                    ids.push(d.id.clone());
                    batch.push(d);
                }
                i.store.save_batch(&batch).map_err(|e| e.to_string())?;
                for d in batch {
                    i.cancel.insert(d.id.clone(), Cancellation::default());
                    i.documents.insert(0, d);
                }
                drop(i);
                self.kick();
                self.emit();
                return Ok(json!(ids));
            }
            "file_cancel" | "file_delete" => {
                let id = id()?;
                let mut i = self.inner.lock();
                if !i.documents.iter().any(|d| d.id == id) {
                    return Err("Dokumentet findes ikke længere.".into());
                }
                if action == "file_cancel"
                    && !i.documents.iter().any(|d| d.id == id && d.stage.busy())
                {
                    return Err("Filen er ikke under behandling.".into());
                }
                if action == "file_delete" {
                    if let Some(c) = i.cancel.get(&format!("ai:{id}")) {
                        c.cancel();
                    }
                }
                if let Some(c) = i.cancel.get(id) {
                    c.cancel();
                }
                if action == "file_delete" {
                    if !i.documents.iter().any(|d| d.id == id && d.stage.busy()) {
                        if let Err(error) = std::fs::remove_file(self.path(id)) {
                            if error.kind() != std::io::ErrorKind::NotFound {
                                return Err("Lydkopien er stadig i brug og kunne ikke slettes. Stop afspilningen og prøv igen.".into());
                            }
                        }
                    }
                    i.store.delete(id).map_err(|e| e.to_string())?;
                    i.documents.retain(|d| d.id != id);
                } else if let Some(mut d) = i.documents.iter().find(|d| d.id == id).cloned() {
                    if d.stage.busy() {
                        d.stage = FileStage::Cancelled;
                        d.error = None;
                        Self::save(&mut i, d)?;
                    }
                }
                drop(i);
                let _ = std::fs::remove_file(self.path(id));
                self.emit();
            }
            "file_retry" => {
                let id = id()?;
                let mut i = self.inner.lock();
                let mut d = i
                    .documents
                    .iter()
                    .find(|d| d.id == id)
                    .cloned()
                    .ok_or("Filen findes ikke længere.")?;
                if d.stage.busy() {
                    return Err("Filen er allerede i kø.".into());
                }
                if settings.strict_local && d.engine.locality != Locality::OnDevice {
                    return Err(
                        "Genprøvning kræver cloud, men kun lokal behandling er slået til.".into(),
                    );
                }
                if d.stage == FileStage::Ready {
                    return Err("Denne fil er allerede færdig.".into());
                }
                d.stage = FileStage::Queued;
                d.error = None;
                d.completed_seconds = 0.;
                i.cancel.insert(id.into(), Cancellation::default());
                Self::save(&mut i, d)?;
                drop(i);
                self.kick();
                self.emit();
            }
            "file_persist" => {
                let mut i = self.inner.lock();
                let mut d = i
                    .documents
                    .iter()
                    .find(|d| Some(d.id.as_str()) == p["id"].as_str())
                    .cloned()
                    .ok_or("Dokumentet mangler.")?;
                if d.transcript.is_none() {
                    return Err("Der er ingen tekst at gemme endnu.".into());
                }
                d.stage = FileStage::Ready;
                d.error = None;
                Self::save(&mut i, d)?;
                drop(i);
                self.emit();
            }
            "file_save" => {
                let id = id()?;
                let revision = p["revision"].as_u64().ok_or("Revision mangler.")?;
                let text = p["text"].as_str().ok_or("Teksten mangler.")?;
                if text.len() > 4_000_000 {
                    return Err("Teksten overstiger dokumentgrænsen.".into());
                }
                let mut i = self.inner.lock();
                let mut d = i
                    .documents
                    .iter()
                    .find(|d| d.id == id)
                    .cloned()
                    .ok_or("Dokumentet mangler.")?;
                if d.revision != revision {
                    return Err("Dokumentet er ændret et andet sted. Din kladde er bevaret; indlæs dokumentet igen.".into());
                }
                let original = &d
                    .transcript
                    .as_ref()
                    .ok_or("Transskriptionen er ikke klar.")?
                    .text;
                d.edited_text = if text == original {
                    None
                } else {
                    Some(text.into())
                };
                d.revision += 1;
                Self::save(&mut i, d)?;
                drop(i);
                self.emit();
                return Ok(json!({"revision":revision+1}));
            }
            "file_rename" => {
                let title = p["title"]
                    .as_str()
                    .filter(|s| !s.trim().is_empty() && s.len() < 500)
                    .ok_or("Skriv en titel.")?;
                self.update(id()?, |d| d.title = title.trim().into())?;
                self.emit();
            }
            "file_ai_cancel" => {
                if let Some(c) = self.inner.lock().cancel.get(&format!("ai:{}", id()?)) {
                    c.cancel();
                }
            }
            "file_ai" => {
                let id = id()?;
                let prompt = p["prompt"]
                    .as_str()
                    .filter(|s| !s.trim().is_empty() && s.len() < 16000)
                    .ok_or("Skriv instruktioner til AI.")?;
                let engine = settings
                    .engines
                    .iter()
                    .find(|e| {
                        Some(e.id.as_str()) == p["engine"].as_str() && e.kind == EngineKind::Text
                    })
                    .cloned()
                    .ok_or("Vælg en tekstmodel.")?;
                if settings.strict_local && engine.locality != Locality::OnDevice {
                    return Err("Kun lokal behandling er slået til.".into());
                }
                let d = self
                    .inner
                    .lock()
                    .documents
                    .iter()
                    .find(|d| d.id == id)
                    .cloned()
                    .ok_or("Dokumentet mangler.")?;
                if d.transcript.is_none() {
                    return Err("Transskriptionen er ikke klar.".into());
                }
                let tag = format!("ai:{id}");
                let c = Cancellation::default();
                {
                    let mut i = self.inner.lock();
                    if i.cancel.contains_key(&tag) {
                        return Err("AI arbejder allerede på dokumentet.".into());
                    }
                    i.cancel.insert(tag.clone(), c.clone());
                }
                let work = (|| {
                    let key = crate::service::engine_secret(&engine)?;
                    let plan = TextPlan {
                        engine: engine.clone(),
                        system_prompt: prompt.into(),
                        app_specific: false,
                    };
                    let output = tauri::async_runtime::block_on(self.engines.text(
                        &plan,
                        d.text(),
                        key.as_deref(),
                        &c,
                    ))?;
                    let mut i = self.inner.lock();
                    c.check()?;
                    let mut current = i
                        .documents
                        .iter()
                        .find(|v| v.id == id)
                        .cloned()
                        .ok_or("Dokumentet er slettet.")?;
                    if current.revision != d.revision {
                        return Err("Teksten blev rettet under AI-bearbejdningen. Prøv igen med den nye tekst.".into());
                    }
                    current.ai_text = Some(output);
                    current.ai_engine = Some(engine);
                    Self::save(&mut i, current)
                })();
                self.inner.lock().cancel.remove(&tag);
                self.emit();
                work?;
            }
            "file_speaker" => {
                let speaker = p["speaker"].as_str().ok_or("Taler mangler.")?;
                let name = p["name"]
                    .as_str()
                    .filter(|s| !s.trim().is_empty() && s.len() < 200)
                    .ok_or("Skriv et navn.")?;
                self.update(id()?, |d| {
                    d.speakers.insert(speaker.into(), name.trim().into());
                })?;
                self.emit();
            }
            "file_copy" | "file_export" => {
                let mut d = self
                    .inner
                    .lock()
                    .documents
                    .iter()
                    .find(|d| Some(d.id.as_str()) == p["id"].as_str())
                    .cloned()
                    .ok_or("Dokumentet mangler.")?;
                let format = p["format"].as_str().unwrap_or("txt");
                let original = p["original"] == true;
                let unsaved_draft = !original && p["draft"].is_string();
                if unsaved_draft {
                    let text = p["draft"].as_str().unwrap();
                    if text.len() > 4_000_000 {
                        return Err("Kladde overstiger tekstgrænsen.".into());
                    }
                    d.edited_text = Some(text.into());
                }
                let text = if p["ai"] == true {
                    let text = d.ai_text.clone().ok_or("Ingen AI-tekst endnu.")?;
                    match format {"txt"=>text,"md"=>format!("# {} — AI\n\n{}\n",d.title,text),"json"=>d.export("json",true).map_err(|e|e.to_string())?,_=>return Err("AI-tekst har ingen tidskoder. Eksportér som tekst, Markdown eller JSON.".into())}
                } else {
                    d.export(format, original).map_err(|e| e.to_string())?
                };
                let text = if format == "json" && unsaved_draft {
                    let mut json: Value = serde_json::from_str(&text)
                        .map_err(|_| "Dokumentet kunne ikke eksporteres.")?;
                    json["unsaved_draft"] = json!(true);
                    json.to_string()
                } else {
                    text
                };
                if action == "file_copy" {
                    vara_platform::copy(&text)?;
                } else {
                    let safe = d
                        .title
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() || " -_".contains(c) {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect::<String>();
                    let path = rfd::FileDialog::new()
                        .set_file_name(format!("{safe}.{format}"))
                        .add_filter("Transskription", &[format])
                        .save_file();
                    if let Some(path) = path {
                        if same_file::is_same_file(&path, &d.source_path).unwrap_or(false)
                            || same_file::is_same_file(&path, self.path(&d.id)).unwrap_or(false)
                        {
                            return Err(
                                "Eksporten må ikke overskrive originalen eller lydkopien.".into()
                            );
                        }
                        let target = path.canonicalize().unwrap_or(path.clone());
                        if target
                            == PathBuf::from(&d.source_path)
                                .canonicalize()
                                .unwrap_or_default()
                            || target == self.path(&d.id).canonicalize().unwrap_or_default()
                        {
                            return Err(
                                "Eksporten må ikke overskrive kildefilen eller arbejdslyden."
                                    .into(),
                            );
                        }
                        std::fs::write(path, text).map_err(|_| "Eksporten kunne ikke gemmes.")?;
                    } else {
                        return Ok(json!(false));
                    }
                }
            }
            _ => return Err("Ukendt filhandling.".into()),
        }
        Ok(Value::Null)
    }
    fn kick(self: &Arc<Self>) {
        let mut i = self.inner.lock();
        if i.worker {
            return;
        }
        i.worker = true;
        drop(i);
        let files = self.clone();
        tauri::async_runtime::spawn_blocking(move || files.run());
    }
    fn run(self: Arc<Self>) {
        loop {
            let next = {
                let mut i = self.inner.lock();
                let next = i
                    .documents
                    .iter()
                    .rev()
                    .find(|d| d.stage == FileStage::Queued)
                    .cloned();
                if let Some(mut d) = next {
                    d.stage = FileStage::Decoding;
                    let c = i.cancel.entry(d.id.clone()).or_default().clone();
                    if Self::save(&mut i, d.clone()).is_err() {
                        i.worker = false;
                        return;
                    }
                    Some((d, c))
                } else {
                    i.worker = false;
                    None
                }
            };
            let Some((d, c)) = next else {
                return;
            };
            self.emit();
            let result = self.process(&d, &c);
            let _ = std::fs::remove_file(self.root.join(format!("{}.part", d.id)));
            let mut i = self.inner.lock();
            if let Some(mut current) = i.documents.iter().find(|v| v.id == d.id).cloned() {
                if c.check().is_ok() && current.stage != FileStage::Cancelled {
                    match result {
                        Ok((duration, t)) => {
                            current.stage = FileStage::Ready;
                            current.duration = duration;
                            current.completed_seconds = duration;
                            let mut names = BTreeMap::new();
                            for segment in &t.segments {
                                if let Some(speaker) = &segment.speaker {
                                    if !names.contains_key(speaker) {
                                        names.insert(
                                            speaker.clone(),
                                            format!("Taler {}", names.len() + 1),
                                        );
                                    }
                                }
                            }
                            current.speakers = names;
                            current.transcript = Some(t);
                            current.error = None;
                        }
                        Err(error) => {
                            current.stage = FileStage::Failed;
                            current.error = Some(error);
                        }
                    }
                    if let Err(error) = Self::save(&mut i, current.clone()) {
                        current.stage = FileStage::Failed;
                        current.error=Some("Resultatet kunne ikke gemmes på disk. Teksten er bevaret i denne kørsel: kopiér, eksportér eller prøv at gemme igen, før Vara lukkes.".into());
                        if let Some(old) = i.documents.iter_mut().find(|v| v.id == current.id) {
                            *old = current;
                        }
                        let _ = self.app.emit("vara:file-error", error);
                    }
                } else {
                    let _ = std::fs::remove_file(self.path(&d.id));
                }
            } else {
                let _ = std::fs::remove_file(self.path(&d.id));
            }
            i.cancel.remove(&d.id);
            drop(i);
            self.emit();
        }
    }
    fn process(&self, d: &TranscriptionDocument, c: &Cancellation) -> Result<(f64, Transcript)> {
        let partial = self.root.join(format!("{}.part", d.id));
        let duration = if let Ok(duration) =
            vara_engine::media::normalized_duration(&self.path(&d.id))
        {
            duration
        } else {
            let duration =
                vara_engine::media::normalize(std::path::Path::new(&d.source_path), &partial, c)?;
            c.check()?;
            std::fs::rename(&partial, self.path(&d.id))
                .map_err(|_| "Lydkopien kunne ikke færdiggøres.")?;
            duration
        };
        let waveform = vara_engine::media::waveform(&self.path(&d.id), c)?;
        self.update_active(&d.id, c, |d| {
            d.waveform = waveform;
            d.duration = duration;
            d.stage = FileStage::Transcribing;
        })?;
        self.emit();
        let secret = crate::service::engine_secret(&d.engine)?;
        let t = self.engines.transcribe_file(
            &d.engine,
            &self.path(&d.id),
            d.language.as_deref(),
            secret.as_deref(),
            c,
            &|seconds| {
                if c.check().is_ok() {
                    let _ = self.update_active(&d.id, c, |d| d.completed_seconds = seconds);
                    self.emit();
                }
            },
        )?;
        c.check()?;
        t.validate(duration).map_err(|e| e.to_string())?;
        Ok((duration, t))
    }
}

fn document_id(p: &Value) -> Result<&str> {
    p["id"]
        .as_str()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .ok_or_else(|| "Ugyldigt dokument-id.".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_ids_are_opaque_and_cannot_select_paths() {
        assert!(document_id(&json!({"id":"../../original"})).is_err());
        assert!(document_id(&json!({"id":"/tmp/recording"})).is_err());
        assert!(document_id(&json!({})).is_err());
        let id = uuid::Uuid::new_v4().to_string();
        let value = json!({"id":id});
        assert_eq!(document_id(&value).unwrap(), id);
    }
}
