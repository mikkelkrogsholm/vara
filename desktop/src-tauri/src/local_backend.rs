//! Internal local file-service protocol. This is not the public agent CLI.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

// v2 prevents old owners silently ignoring the durable Who request fields.
pub const VERSION: u32 = 2;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Ping,
    Models,
    List,
    Submit {
        request: vara_app::SubmitRequest,
    },
    Add {
        paths: Vec<String>,
        engine: String,
        language: Option<String>,
    },
    Status {
        id: String,
    },
    Result {
        id: String,
    },
    Cancel {
        id: String,
    },
    Export {
        id: String,
        format: String,
        path: PathBuf,
        overwrite: bool,
        #[serde(default)]
        original: bool,
    },
    Retry {
        id: String,
    },
    Delete {
        id: String,
    },
    Persist {
        id: String,
    },
    Save {
        id: String,
        revision: u64,
        text: String,
    },
    Rename {
        id: String,
        title: String,
    },
    Speaker {
        id: String,
        speaker: String,
        name: String,
    },
    Ai {
        id: String,
        prompt: String,
        engine: String,
    },
    AiCancel {
        id: String,
    },
    WhoStatus,
    Who {
        id: String,
        known_speaker_count: Option<u32>,
    },
    WhoDownload {
        acknowledge_license: bool,
    },
    WhoCancel {
        id: Option<String>,
        #[serde(default)]
        download: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub operation: Operation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub version: u32,
    pub instance: String,
    pub revision: u64,
    pub notice: Option<Notice>,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
}
/// Most recent asynchronous Files error, scoped to the response instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notice {
    pub revision: u64,
    pub message: vara_core::UserMessage,
}
/// Transport errors carry stable backend codes; application errors include a
/// sanitized, catalogued UserMessage with its numeric parameters and causes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: String,
    pub message: Option<vara_core::UserMessage>,
}

/// The same profile as Tauri, unless its explicit override is set.
pub fn default_profile() -> Result<PathBuf, String> {
    std::env::var_os("VARA_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::data_dir().map(|d| d.join("dk.vara.desktop")))
        .ok_or_else(|| "backend.profile_unavailable".into())
}

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::{
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        os::{
            fd::AsRawFd,
            unix::{
                fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
                net::{UnixListener, UnixStream},
            },
        },
        sync::{
            Arc,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        },
        time::{Duration, Instant},
    };
    const MAX_REQUEST: usize = 32 * 1024 * 1024;
    const MAX_RESPONSE: usize = 128 * 1024 * 1024;
    const TIMEOUT: Duration = Duration::from_secs(30);
    const IDLE: Duration = Duration::from_secs(60);
    fn err(code: &str) -> String {
        code.into()
    }
    fn uid() -> u32 {
        unsafe { libc::geteuid() }
    }
    fn private(m: &fs::Metadata) -> bool {
        m.uid() == uid() && m.mode() & 0o077 == 0
    }
    fn dir(path: &Path) -> Result<(), String> {
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(err("backend.profile_unavailable")),
        }
        let m = fs::symlink_metadata(path).map_err(|_| err("backend.permissions"))?;
        if !m.file_type().is_dir() || !private(&m) {
            return Err(err("backend.permissions"));
        }
        Ok(())
    }
    fn profile(path: &Path) -> Result<PathBuf, String> {
        // Resolve existing ancestors, reject a symlink as the profile itself.
        if !path.exists() {
            let parent = path
                .parent()
                .ok_or_else(|| err("backend.profile_unavailable"))?;
            fs::create_dir_all(parent).map_err(|_| err("backend.profile_unavailable"))?;
        }
        // Existing Tauri profiles were commonly created with the process umask.
        // Tighten an owned directory through an O_NOFOLLOW descriptor before use.
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(err("backend.profile_unavailable")),
        }
        let f = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open(path)
            .map_err(|_| err("backend.permissions"))?;
        let m = f.metadata().map_err(|_| err("backend.permissions"))?;
        if !m.is_dir() || m.uid() != uid() {
            return Err(err("backend.permissions"));
        }
        if m.mode() & 0o077 != 0 {
            f.set_permissions(fs::Permissions::from_mode(0o700))
                .map_err(|_| err("backend.permissions"))?;
        }
        fs::canonicalize(path).map_err(|_| err("backend.profile_unavailable"))
    }
    fn paths(root: &Path) -> Result<(PathBuf, PathBuf), String> {
        let base = PathBuf::from(format!("/tmp/vara-{}", uid()));
        dir(&base)?;
        let hash = Sha256::digest(root.as_os_str().as_encoded_bytes());
        let name: String = hash[..16].iter().map(|b| format!("{b:02x}")).collect();
        Ok((
            base.join(format!("{name}.sock")),
            base.join(format!("{name}.lock")),
        ))
    }
    fn lock(path: &Path) -> Result<File, String> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|_| err("backend.permissions"))?;
        let m = f.metadata().map_err(|_| err("backend.permissions"))?;
        if !m.is_file() || !private(&m) || m.nlink() != 1 {
            return Err(err("backend.permissions"));
        }
        f.try_lock().map_err(|_| err("backend.already_running"))?;
        Ok(f)
    }
    fn peer_is(s: &UnixStream, expected: u32) -> bool {
        let (mut user, mut group) = (0, 0);
        unsafe { libc::getpeereid(s.as_raw_fd(), &mut user, &mut group) == 0 && user == expected }
    }
    fn peer(s: &UnixStream) -> bool {
        peer_is(s, uid())
    }
    fn connect(path: &Path) -> Result<UnixStream, String> {
        let m = fs::symlink_metadata(path).map_err(|_| err("backend.unavailable"))?;
        if !m.file_type().is_socket() || !private(&m) {
            return Err(err("backend.permissions"));
        }
        let s = UnixStream::connect(path).map_err(|_| err("backend.unavailable"))?;
        if !peer(&s) {
            return Err(err("backend.peer_denied"));
        }
        s.set_nonblocking(true)
            .map_err(|_| err("backend.transport"))?;
        Ok(s)
    }
    fn wait_ready(s: &UnixStream, events: libc::c_short, deadline: Instant) -> Result<(), String> {
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| err("backend.timeout"))?;
            let millis = remaining.as_millis().min(i32::MAX as u128) as i32;
            if millis == 0 {
                return Err(err("backend.timeout"));
            }
            let mut fd = libc::pollfd {
                fd: s.as_raw_fd(),
                events,
                revents: 0,
            };
            let n = unsafe { libc::poll(&mut fd, 1, millis) };
            if n > 0 {
                return Ok(());
            }
            if n == 0 {
                return Err(err("backend.timeout"));
            }
            if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
                return Err(err("backend.transport"));
            }
        }
    }
    fn read_exact_until(
        s: &mut UnixStream,
        mut out: &mut [u8],
        deadline: Instant,
    ) -> Result<(), String> {
        while !out.is_empty() {
            wait_ready(s, libc::POLLIN, deadline)?;
            match s.read(out) {
                Ok(0) => return Err(err("backend.malformed")),
                Ok(n) => out = &mut out[n..],
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    continue;
                }
                Err(_) => return Err(err("backend.transport")),
            }
        }
        Ok(())
    }
    fn read_frame(s: &mut UnixStream, limit: usize) -> Result<Vec<u8>, String> {
        read_frame_wait(s, limit, false)
    }
    // Long operations may run for minutes; the frame deadline starts only once
    // the first response byte arrives. Cancellation uses a separate connection.
    fn read_frame_wait(
        s: &mut UnixStream,
        limit: usize,
        wait_for_work: bool,
    ) -> Result<Vec<u8>, String> {
        let mut header = [0; 4];
        if wait_for_work {
            loop {
                let mut fd = libc::pollfd {
                    fd: s.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                let n = unsafe { libc::poll(&mut fd, 1, -1) };
                if n > 0 {
                    break;
                }
                if n < 0
                    && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
                {
                    return Err(err("backend.transport"));
                }
            }
        }
        let deadline = Instant::now() + TIMEOUT;
        read_exact_until(s, &mut header, deadline)?;
        let len = u32::from_be_bytes(header) as usize;
        if len == 0 || len > limit {
            return Err(err("backend.frame_limit"));
        }
        let mut data = vec![0; len];
        read_exact_until(s, &mut data, deadline)?;
        Ok(data)
    }
    fn write_frame(s: &mut UnixStream, data: &[u8], limit: usize) -> Result<(), String> {
        if data.len() > limit {
            return Err(err("backend.frame_limit"));
        }
        let deadline = Instant::now() + TIMEOUT;
        for mut bytes in [&(data.len() as u32).to_be_bytes()[..], data] {
            while !bytes.is_empty() {
                wait_ready(s, libc::POLLOUT, deadline)?;
                match s.write(bytes) {
                    Ok(0) => return Err(err("backend.transport")),
                    Ok(n) => bytes = &bytes[n..],
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                        ) =>
                    {
                        continue;
                    }
                    Err(_) => return Err(err("backend.transport")),
                }
            }
        }
        Ok(())
    }
    use vara_app::{safe_error, sanitize};
    struct State {
        root: PathBuf,
        files: Arc<vara_app::Files>,
        instance: String,
        revision: Arc<AtomicU64>,
        notice: Arc<std::sync::Mutex<Option<Notice>>>,
        active: AtomicUsize,
    }
    impl State {
        fn run(&self, operation: Operation) -> Result<Value, String> {
            #[cfg(feature = "test-harness")]
            if matches!(&operation, Operation::Status { .. }) {
                if let Some(ms) = std::env::var("VARA_TEST_BACKEND_DELAY_STATUS_MS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    std::thread::sleep(Duration::from_millis(ms.min(45_000)));
                }
            }
            let settings = vara_core::Store::read_settings(&self.root.join("vara.sqlite3"))
                .map_err(|e| e.wire())?;
            let args = match operation {
                Operation::Ping => return Ok(json!({"ready":true})),
                Operation::Models => {
                    let engines = vara_engine::Engines::new(self.root.join("models"))?;
                    return Ok(
                        json!({"default_speech_engine":settings.speech_engine,"strict_local":settings.strict_local,"models": settings.engines.iter().map(|e| json!({"id":e.id,"label":e.label,"kind":e.kind,"locality":e.locality,"timing_capability":e.file_timing_capability(),"ready":if e.locality == vara_core::Locality::OnDevice { Some(engines.ready(&e.id)) } else { None }})).collect::<Vec<_>>() }),
                    );
                }
                Operation::Submit { request } => {
                    let receipt = self.files.submit(request, &settings)?;
                    #[cfg(feature = "test-harness")]
                    if let Some(ms) = std::env::var("VARA_TEST_BACKEND_DELAY_SUBMIT_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                    {
                        std::thread::sleep(Duration::from_millis(ms.min(45_000)));
                    }
                    return serde_json::to_value(receipt)
                        .map_err(|_| err("backend.operation_failed"));
                }
                Operation::Status { id } => {
                    return serde_json::to_value(self.files.status(&id)?)
                        .map_err(|_| err("backend.operation_failed"));
                }
                Operation::Result { id } => {
                    return serde_json::to_value(self.files.result(&id)?)
                        .map_err(|_| err("backend.operation_failed"));
                }
                Operation::Add {
                    paths,
                    engine,
                    language,
                } => (
                    "file_add",
                    json!({"paths":paths,"engine":engine,"language":language}),
                ),
                Operation::Export {
                    id,
                    format,
                    path,
                    overwrite,
                    original,
                } => {
                    self.files
                        .export_to(&id, &format, &path, overwrite, original)?;
                    return Ok(Value::Null);
                }
                Operation::Cancel { id } => ("file_cancel", json!({"id":id})),
                Operation::List => ("file_list", json!({})),
                Operation::Retry { id } => ("file_retry", json!({"id":id})),
                Operation::Delete { id } => ("file_delete", json!({"id":id})),
                Operation::Persist { id } => ("file_persist", json!({"id":id})),
                Operation::Save { id, revision, text } => (
                    "file_save",
                    json!({"id":id,"revision":revision,"text":text}),
                ),
                Operation::Rename { id, title } => ("file_rename", json!({"id":id,"title":title})),
                Operation::Speaker { id, speaker, name } => (
                    "file_speaker",
                    json!({"id":id,"speaker":speaker,"name":name}),
                ),
                Operation::Ai { id, prompt, engine } => {
                    ("file_ai", json!({"id":id,"prompt":prompt,"engine":engine}))
                }
                Operation::AiCancel { id } => ("file_ai_cancel", json!({"id":id})),
                Operation::WhoStatus => ("file_who_status", json!({})),
                Operation::Who {
                    id,
                    known_speaker_count,
                } => (
                    "file_who",
                    json!({"id":id,"known_speaker_count":known_speaker_count}),
                ),
                Operation::WhoDownload {
                    acknowledge_license,
                } => (
                    "file_who_download",
                    json!({"acknowledge_license":acknowledge_license}),
                ),
                Operation::WhoCancel { id, download } => {
                    ("file_who_cancel", json!({"id":id,"download":download}))
                }
            };
            self.files.command(args.0, args.1, &settings)
        }
        fn handle(&self, mut s: UnixStream) {
            if !peer(&s) {
                return;
            }
            let result = read_frame(&mut s, MAX_REQUEST).and_then(|bytes| {
                // Parse version before the operation, so newer variants get a version error.
                let raw: Value =
                    serde_json::from_slice(&bytes).map_err(|_| err("backend.malformed"))?;
                if raw.get("version").and_then(Value::as_u64) != Some(VERSION as u64) {
                    return Err(err("backend.version_mismatch"));
                }
                let request: Request =
                    serde_json::from_value(raw).map_err(|_| err("backend.malformed"))?;
                self.run(request.operation).map_err(|e| safe_error(&e))
            });
            let (mut value, error) = match result {
                Ok(v) => (Some(v), None),
                Err(e) => {
                    let message = vara_core::UserMessage::decode(&e);
                    let code = message.as_ref().map(|m| m.code.clone()).unwrap_or(e);
                    (None, Some(RpcError { code, message }))
                }
            };
            if let Some(v) = value.as_mut() {
                sanitize(v);
            }
            let notice_guard = self.notice.lock().unwrap_or_else(|e| e.into_inner());
            let response = Response {
                version: VERSION,
                instance: self.instance.clone(),
                revision: self.revision.load(Ordering::SeqCst),
                notice: notice_guard.clone(),
                result: value,
                error,
            };
            drop(notice_guard);
            if let Ok(data) = serde_json::to_vec(&response) {
                let _ = write_frame(&mut s, &data, MAX_RESPONSE);
            }
        }
    }
    fn resources() -> Option<PathBuf> {
        #[cfg(feature = "test-harness")]
        if let Some(path) = std::env::var_os("VARA_RESOURCE_DIR") {
            return Some(PathBuf::from(path));
        }
        let executable = std::env::current_exe().ok()?;
        let macos = executable.parent()?;
        if macos.file_name()? != "MacOS" {
            return None;
        }
        Some(macos.parent()?.join("Resources"))
    }
    pub fn serve(root: &Path) -> Result<(), String> {
        let root = profile(root)?;
        let (socket, lock_path) = paths(&root)?;
        let _lock = lock(&lock_path)?;
        if let Ok(m) = fs::symlink_metadata(&socket) {
            if !m.file_type().is_socket() || !private(&m) {
                return Err(err("backend.permissions"));
            }
            match UnixStream::connect(&socket) {
                Ok(_) => return Err(err("backend.already_running")),
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {}
                Err(_) => return Err(err("backend.unavailable")),
            }
            fs::remove_file(&socket).map_err(|_| err("backend.transport"))?;
        }
        let revision = Arc::new(AtomicU64::new(0));
        let counter = revision.clone();
        let notice = Arc::new(std::sync::Mutex::new(None));
        let notification = notice.clone();
        let notify = move |error: Option<String>| {
            let mut last = notification.lock().unwrap_or_else(|e| e.into_inner());
            let rev = counter.fetch_add(1, Ordering::SeqCst) + 1;
            if let Some(error) = error {
                let message = vara_core::UserMessage::decode(&safe_error(&error)).unwrap();
                *last = Some(Notice {
                    revision: rev,
                    message,
                });
            }
        };
        #[cfg(feature = "test-harness")]
        if std::env::var_os("VARA_TEST_BACKEND_NOTICE").is_some() {
            notify(Some(
                vara_core::UserMessage::new("file.result_unsaved").wire(),
            ));
        }
        let files = vara_app::Files::new(
            vara_app::Host {
                resource_dir: resources(),
                secret: Box::new(|engine| {
                    #[cfg(feature = "test-harness")]
                    if std::env::var_os("VARA_TEST_BACKEND_NO_SECRET").is_some() {
                        let delay = if engine.kind == vara_core::EngineKind::Text {
                            "VARA_TEST_BACKEND_DELAY_TEXT_MS"
                        } else {
                            "VARA_TEST_BACKEND_DELAY_SECRET_MS"
                        };
                        if let Ok(ms) = std::env::var(delay) {
                            if let Ok(ms) = ms.parse::<u64>() {
                                std::thread::sleep(Duration::from_millis(ms.min(45_000)));
                            }
                        }
                        return Ok(None);
                    }
                    crate::service::engine_secret(engine)
                }),
                notify: Box::new(notify),
            },
            root.clone(),
        )
        .map_err(|e| safe_error(&e))?;
        // This worker has not started other threads yet. Bind with private permissions
        // from the first instant; chmod after bind alone leaves a connection race.
        let old_umask = unsafe { libc::umask(0o077) };
        let bound = UnixListener::bind(&socket);
        unsafe {
            libc::umask(old_umask);
        }
        let listener = bound.map_err(|_| err("backend.transport"))?;
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))
            .map_err(|_| err("backend.permissions"))?;
        listener
            .set_nonblocking(true)
            .map_err(|_| err("backend.transport"))?;
        let state = Arc::new(State {
            root,
            files,
            instance: uuid::Uuid::new_v4().to_string(),
            revision,
            notice,
            active: AtomicUsize::new(0),
        });
        let mut last = Instant::now();
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }
                    last = Instant::now();
                    state.active.fetch_add(1, Ordering::SeqCst);
                    let s = state.clone();
                    std::thread::spawn(move || {
                        s.handle(stream);
                        s.active.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if state.active.load(Ordering::SeqCst) > 0 || state.files.busy() {
                        last = Instant::now();
                    }
                    #[cfg(feature = "test-harness")]
                    let idle = std::env::var("VARA_TEST_BACKEND_IDLE_MS")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .map(Duration::from_millis)
                        .unwrap_or(IDLE);
                    #[cfg(not(feature = "test-harness"))]
                    let idle = IDLE;
                    if last.elapsed() >= idle {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(_) => return Err(err("backend.transport")),
            }
        }
        fs::remove_file(socket).map_err(|_| err("backend.transport"))?;
        Ok(())
    }
    pub fn call(root: &Path, operation: Operation, executable: &Path) -> Result<Response, String> {
        let root = profile(root)?;
        let (socket, _) = paths(&root)?;
        let mut stream = match connect(&socket) {
            Ok(s) => s,
            Err(e) if e == "backend.unavailable" => {
                use std::os::unix::process::CommandExt;
                let mut child = std::process::Command::new(executable)
                    .process_group(0)
                    .arg("--vara-internal-file-worker")
                    .arg(&root)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map_err(|_| err("backend.start_failed"))?;
                // Reap contenders and the eventually idle worker while a long-lived
                // GUI remains open; dropping the GUI never signals the worker.
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                let until = Instant::now() + Duration::from_secs(15);
                loop {
                    match connect(&socket) {
                        Ok(s) => break s,
                        Err(e) if e == "backend.unavailable" && Instant::now() < until => {
                            std::thread::sleep(Duration::from_millis(50))
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e),
        };
        let wait_for_work = matches!(
            &operation,
            Operation::Ai { .. } | Operation::Who { .. } | Operation::WhoDownload { .. }
        );
        let data = serde_json::to_vec(&Request {
            version: VERSION,
            operation,
        })
        .map_err(|_| err("backend.malformed"))?;
        write_frame(&mut stream, &data, MAX_REQUEST)?;
        let bytes = read_frame_wait(&mut stream, MAX_RESPONSE, wait_for_work)?;
        let response: Response =
            serde_json::from_slice(&bytes).map_err(|_| err("backend.response_invalid"))?;
        if response.version != VERSION {
            return Err(err("backend.version_mismatch"));
        }
        Ok(response)
    }

    #[cfg(all(test, feature = "test-harness"))]
    mod tests {
        use super::*;
        fn executable() -> PathBuf {
            PathBuf::from(
                std::env::var_os("VARA_TEST_BACKEND_EXECUTABLE").expect("set backend executable"),
            )
        }
        fn configure(cmd: &mut std::process::Command) {
            cmd.env("VARA_TEST_BACKEND_NO_SECRET", "1")
                .env("VARA_TEST_BACKEND_IDLE_MS", "1800");
        }
        // Spawned by the parent test, so concurrent clients have independent processes.
        #[test]
        fn client_process() {
            let Some(root) = std::env::var_os("VARA_TEST_CLIENT_PROFILE") else {
                return;
            };
            let response = call(Path::new(&root), Operation::Ping, &executable()).unwrap();
            std::fs::write(
                std::env::var_os("VARA_TEST_CLIENT_OUTPUT").unwrap(),
                response.instance,
            )
            .unwrap();
        }
        fn invoke(root: &Path, op: Operation) -> Response {
            let label = format!("{op:?}");
            call(root, op, &executable()).unwrap_or_else(|e| panic!("{label}: {e}"))
        }
        fn raw(root: &Path, data: &[u8]) -> Response {
            let (socket, _) = paths(&fs::canonicalize(root).unwrap()).unwrap();
            let mut stream = connect(&socket).unwrap();
            write_frame(&mut stream, data, MAX_REQUEST).unwrap();
            serde_json::from_slice(&read_frame(&mut stream, MAX_RESPONSE).unwrap()).unwrap()
        }
        fn wait_stopped(root: &Path) {
            let (socket, _) = paths(&fs::canonicalize(root).unwrap()).unwrap();
            let end = Instant::now() + Duration::from_secs(12);
            while socket.exists() {
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(50));
            }
            let (_, lock_path) = paths(&fs::canonicalize(root).unwrap()).unwrap();
            loop {
                if let Ok(guard) = lock(&lock_path) {
                    drop(guard);
                    break;
                }
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        fn wav(path: &Path) {
            let samples = vec![0u8; 32000];
            let mut w = Vec::new();
            w.extend_from_slice(b"RIFF");
            w.extend_from_slice(&(36u32 + samples.len() as u32).to_le_bytes());
            w.extend_from_slice(b"WAVEfmt ");
            w.extend_from_slice(&16u32.to_le_bytes());
            w.extend_from_slice(&1u16.to_le_bytes());
            w.extend_from_slice(&1u16.to_le_bytes());
            w.extend_from_slice(&16000u32.to_le_bytes());
            w.extend_from_slice(&32000u32.to_le_bytes());
            w.extend_from_slice(&2u16.to_le_bytes());
            w.extend_from_slice(&16u16.to_le_bytes());
            w.extend_from_slice(b"data");
            w.extend_from_slice(&(samples.len() as u32).to_le_bytes());
            w.extend_from_slice(&samples);
            fs::write(path, w).unwrap();
        }
        #[test]
        fn signal_client_process() {
            let Some(root) = std::env::var_os("VARA_TEST_SIGNAL_PROFILE") else {
                return;
            };
            let response = invoke(
                Path::new(&root),
                Operation::Submit {
                    request: vara_app::SubmitRequest {
                        source: PathBuf::from(std::env::var_os("VARA_TEST_SIGNAL_SOURCE").unwrap()),
                        engine_id: Some("openai.speech".into()),
                        language: None,
                        request_id: Some("signal-test".into()),
                        diarize: false,
                        speakers: None,
                    },
                },
            );
            assert!(response.error.is_none());
            fs::write(
                std::env::var_os("VARA_TEST_SIGNAL_OUTPUT").unwrap(),
                serde_json::to_vec(&response).unwrap(),
            )
            .unwrap();
            loop {
                std::thread::park();
            }
        }
        #[test]
        fn client_group_sigint_does_not_interrupt_backend_job() {
            use std::os::unix::process::{CommandExt, ExitStatusExt};
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("profile");
            fs::create_dir(&root).unwrap();
            let source = temp.path().join("signal.wav");
            wav(&source);
            let receipt = temp.path().join("receipt.json");
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .process_group(0)
                .args([
                    "--exact",
                    "local_backend::mac::tests::signal_client_process",
                    "--nocapture",
                ])
                .env("VARA_TEST_SIGNAL_PROFILE", &root)
                .env("VARA_TEST_SIGNAL_SOURCE", &source)
                .env("VARA_TEST_SIGNAL_OUTPUT", &receipt)
                .env("VARA_TEST_BACKEND_DELAY_SECRET_MS", "6000")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            configure(&mut command);
            let mut client = command.spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            while !receipt.exists() {
                assert!(
                    client.try_wait().unwrap().is_none(),
                    "fixture client exited early"
                );
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(25));
            }
            let response: Response = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
            let instance = response.instance;
            let id = response.result.unwrap()["job_id"]
                .as_str()
                .unwrap()
                .to_owned();
            assert_eq!(
                unsafe { libc::kill(-(client.id() as i32), libc::SIGINT) },
                0
            );
            assert_eq!(client.wait().unwrap().signal(), Some(libc::SIGINT));
            loop {
                let status = invoke(&root, Operation::Status { id: id.clone() });
                assert_eq!(status.instance, instance, "backend died with client group");
                let doc = status.result.unwrap();
                if doc["stage"] == "failed" {
                    assert_eq!(
                        vara_core::UserMessage::decode(doc["error"].as_str().unwrap())
                            .unwrap()
                            .code,
                        "provider.missing_key"
                    );
                    break;
                }
                assert_ne!(doc["stage"], "cancelled");
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(50));
            }
            wait_stopped(&root);
        }
        #[test]
        fn processes_socket_lifecycle_and_isolation() {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("profile-a");
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
            // Pre-existing stale endpoint is recovered only after taking its lock.
            let canonical = fs::canonicalize(&root).unwrap();
            let (socket, _) = paths(&canonical).unwrap();
            let stale = UnixListener::bind(&socket).unwrap();
            fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).unwrap();
            drop(stale);
            let mut clients = Vec::new();
            for n in 0..8 {
                let output = temp.path().join(format!("client-{n}"));
                let mut child = std::process::Command::new(std::env::current_exe().unwrap());
                child
                    .args([
                        "--exact",
                        "local_backend::mac::tests::client_process",
                        "--nocapture",
                    ])
                    .env("VARA_TEST_CLIENT_PROFILE", &root)
                    .env("VARA_TEST_CLIENT_OUTPUT", &output)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                configure(&mut child);
                clients.push((child.spawn().unwrap(), output));
            }
            let mut ids = Vec::new();
            for (mut child, output) in clients {
                assert!(child.wait().unwrap().success());
                ids.push(fs::read_to_string(output).unwrap());
            }
            assert!(ids.iter().all(|v| v == &ids[0]));
            assert_eq!(fs::metadata(&root).unwrap().mode() & 0o777, 0o700);
            let metadata = fs::symlink_metadata(&socket).unwrap();
            assert!(private(&metadata));
            assert!(metadata.file_type().is_socket());
            let peer_stream = connect(&socket).unwrap();
            assert!(peer(&peer_stream));
            assert!(!peer_is(&peer_stream, uid().wrapping_add(1)));
            drop(peer_stream);
            assert_eq!(
                raw(
                    &root,
                    br#"{"version":999,"operation":{"action":"new_action"}}"#
                )
                .error
                .as_ref()
                .map(|e| e.code.as_str()),
                Some("backend.version_mismatch")
            );
            // A v1 submit must never be accepted by this owner: it could not
            // express the durable Who contract. Mismatch does not stop the owner.
            assert_eq!(
                raw(&root, br#"{"version":1,"operation":{"action":"submit","request":{"source":"/fixture.wav","engine_id":null,"language":null,"request_id":"old"}}}"#)
                    .error.as_ref().map(|e| e.code.as_str()),
                Some("backend.version_mismatch")
            );
            assert!(
                invoke(&root, Operation::List).result.unwrap()["documents"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(invoke(&root, Operation::Ping).instance, ids[0]);
            assert_eq!(
                raw(&root, b"not json")
                    .error
                    .as_ref()
                    .map(|e| e.code.as_str()),
                Some("backend.malformed")
            );
            assert_eq!(
                raw(
                    &root,
                    br#"{"version":2,"operation":{"action":"invoke","command":"secret"}}"#
                )
                .error
                .as_ref()
                .map(|e| e.code.as_str()),
                Some("backend.malformed")
            );
            let second = temp.path().join("profile-b");
            let other = invoke(&second, Operation::Ping);
            assert_ne!(other.instance, ids[0]);
            // The authoritative preferences are read for each request without opening the GUI service.
            let store = vara_core::Store::open(&root.join("vara.sqlite3")).unwrap();
            let mut settings = vara_core::Settings::default();
            settings.strict_local = true;
            store.save_settings(&settings).unwrap();
            assert_eq!(
                invoke(&root, Operation::Models).result.unwrap()["strict_local"],
                true
            );
            settings.strict_local = false;
            store.save_settings(&settings).unwrap();
            assert_eq!(
                invoke(&root, Operation::Models).result.unwrap()["strict_local"],
                false
            );
            drop(store);
            let source = temp.path().join("fixture.wav");
            wav(&source);
            let request = Request {
                version: VERSION,
                operation: Operation::Submit {
                    request: vara_app::SubmitRequest {
                        source: source.clone(),
                        engine_id: Some("openai.speech".into()),
                        language: Some("da".into()),
                        request_id: Some("disconnect".into()),
                        diarize: false,
                        speakers: None,
                    },
                },
            };
            let mut stream = connect(&socket).unwrap();
            write_frame(
                &mut stream,
                &serde_json::to_vec(&request).unwrap(),
                MAX_REQUEST,
            )
            .unwrap();
            drop(stream);
            let end = Instant::now() + Duration::from_secs(15);
            let doc = loop {
                let list = invoke(&root, Operation::List).result.unwrap();
                if let Some(d) = list["documents"].as_array().unwrap().first() {
                    if d["stage"] == "failed" {
                        break d.clone();
                    }
                }
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(50));
            };
            let id = doc["id"].as_str().unwrap().to_owned();
            assert_eq!(
                doc["error"]
                    .as_str()
                    .and_then(vara_core::UserMessage::decode)
                    .unwrap()
                    .code,
                "provider.missing_key"
            );
            assert!(doc["duration"].as_f64().unwrap() > 0.0);
            assert_eq!(
                invoke(&root, Operation::Status { id: id.clone() }).instance,
                ids[0]
            );
            let batch = invoke(
                &root,
                Operation::Add {
                    paths: vec![
                        source.to_string_lossy().into(),
                        temp.path().join("missing.wav").to_string_lossy().into(),
                    ],
                    engine: "openai.speech".into(),
                    language: Some("da".into()),
                },
            );
            assert_eq!(batch.error.unwrap().code, "file.source_missing");
            assert_eq!(
                invoke(&root, Operation::List).result.unwrap()["documents"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                invoke(&second, Operation::List).result.unwrap()["documents"]
                    .as_array()
                    .unwrap()
                    .len(),
                0
            );
            wait_stopped(&root);
            wait_stopped(&second);
            // Emulate persisted interrupted work and legacy raw errors, then start a real new worker.
            let store =
                vara_core::TranscriptionStore::open(&root.join("transcriptions.sqlite3")).unwrap();
            let mut d = store.all().unwrap().remove(0);
            d.stage = vara_core::FileStage::Transcribing;
            d.diarization_error = Some("secret-private-response".into());
            d.engine.endpoint = Some("https://example.invalid/?key=secret-endpoint".into());
            let words = (0..450_000)
                .map(|i| vara_core::TimedWord {
                    id: i.to_string(),
                    text: "synthetic".into(),
                    start: i as f64 * 0.01,
                    end: i as f64 * 0.01 + 0.01,
                    speaker: None,
                })
                .collect();
            d.transcript = Some(vara_core::Transcript {
                text: "synthetic".into(),
                timing: vara_core::Timing::Word,
                timing_source: "fixture".into(),
                segments: vec![vara_core::TranscriptSegment {
                    id: "segment".into(),
                    text: "synthetic".into(),
                    start: Some(0.0),
                    end: Some(1500.0),
                    speaker: None,
                    words,
                }],
            });
            store.save(&d).unwrap();
            drop(store);
            let mut process = std::process::Command::new(executable());
            process
                .arg("--vara-internal-file-worker")
                .arg(&root)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            configure(&mut process);
            let mut process = process.spawn().unwrap();
            let end = Instant::now() + Duration::from_secs(15);
            while !socket.exists() {
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(50));
            }
            let restarted = invoke(&root, Operation::Status { id });
            assert_ne!(restarted.instance, ids[0]);
            let body = restarted.result.unwrap();
            assert_eq!(body["stage"], "failed");
            assert_eq!(
                vara_core::UserMessage::decode(body["error"].as_str().unwrap())
                    .unwrap()
                    .code,
                "file.interrupted"
            );
            assert!(!body.to_string().contains("secret-private-response"));
            assert!(!body.to_string().contains("secret-endpoint"));
            assert_eq!(
                body["transcript"]["segments"][0]["words"]
                    .as_array()
                    .unwrap()
                    .len(),
                450_000
            );
            assert!(serde_json::to_vec(&body).unwrap().len() > MAX_REQUEST);
            wait_stopped(&root);
            assert!(process.wait().unwrap().success());
        }
        #[test]
        fn host_notification_is_visible_to_a_reconnecting_client() {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("notice-profile");
            fs::create_dir(&root).unwrap();
            let mut process = std::process::Command::new(executable());
            process
                .arg("--vara-internal-file-worker")
                .arg(&root)
                .env("VARA_TEST_BACKEND_NOTICE", "1")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            configure(&mut process);
            let mut process = process.spawn().unwrap();
            let (socket, _) = paths(&fs::canonicalize(&root).unwrap()).unwrap();
            let end = Instant::now() + Duration::from_secs(15);
            while !socket.exists() {
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(50));
            }
            let first = invoke(&root, Operation::Ping);
            let notice = first.notice.unwrap();
            assert_eq!(notice.message.code, "file.result_unsaved");
            assert!(notice.revision <= first.revision);
            let next = invoke(&root, Operation::Ping);
            assert_eq!(next.notice.unwrap().revision, notice.revision);
            assert_eq!(next.instance, first.instance);
            wait_stopped(&root);
            assert!(process.wait().unwrap().success());
        }
        #[test]
        fn partial_frames_obey_one_absolute_deadline() {
            let (mut reader, mut writer) = UnixStream::pair().unwrap();
            reader.set_nonblocking(true).unwrap();
            let sender = std::thread::spawn(move || {
                for _ in 0..10 {
                    if writer.write_all(&[1]).is_err() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            });
            let mut bytes = [0; 10];
            let start = Instant::now();
            assert_eq!(
                read_exact_until(&mut reader, &mut bytes, start + Duration::from_millis(55))
                    .unwrap_err(),
                "backend.timeout"
            );
            assert!(start.elapsed() < Duration::from_secs(1));
            drop(reader);
            sender.join().unwrap();
        }
        #[test]
        fn rejects_symlink_and_untrusted_endpoint() {
            let temp = tempfile::tempdir().unwrap();
            let real = temp.path().join("real");
            fs::create_dir(&real).unwrap();
            let link = temp.path().join("link");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            assert!(profile(&link).is_err());
            let root = profile(&real).unwrap();
            let (socket, lock_path) = paths(&root).unwrap();
            std::os::unix::fs::symlink(temp.path().join("elsewhere"), &socket).unwrap();
            assert_eq!(connect(&socket).unwrap_err(), "backend.permissions");
            assert_eq!(serve(&root).unwrap_err(), "backend.permissions");
            fs::remove_file(&socket).unwrap();
            fs::remove_file(&lock_path).unwrap();
            std::os::unix::fs::symlink(temp.path().join("elsewhere"), &lock_path).unwrap();
            assert_eq!(lock(&lock_path).unwrap_err(), "backend.permissions");
            fs::remove_file(&lock_path).unwrap();
            let held = lock(&lock_path).unwrap();
            let live = UnixListener::bind(&socket).unwrap();
            fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(serve(&root).unwrap_err(), "backend.already_running");
            assert!(socket.exists());
            drop(live);
            drop(held);
            fs::remove_file(&socket).unwrap();
            assert!(!safe_error("private-provider-body-sk-test").contains("sk-test"));
            let forged = vara_core::UserMessage::new("private-provider-body-sk-test").wire();
            assert!(!safe_error(&forged).contains("sk-test"));
            let classified = vara_core::UserMessage::new("provider.network")
                .number("secret-sk-test", 123)
                .caused_by(vara_core::UserMessage::new("io.os_code").number("code", 42));
            let cleaned = vara_core::UserMessage::decode(&safe_error(&classified.wire())).unwrap();
            assert!(cleaned.params.is_empty());
            assert_eq!(cleaned.cause.unwrap().params["code"], 42);
        }
    }
}
#[cfg(target_os = "macos")]
pub use mac::{call, serve};
#[cfg(not(target_os = "macos"))]
pub fn call(_: &Path, _: Operation, _: &Path) -> Result<Response, String> {
    Err("backend.unsupported_platform".into())
}
#[cfg(not(target_os = "macos"))]
pub fn serve(_: &Path) -> Result<(), String> {
    Err("backend.unsupported_platform".into())
}

/// Dispatch only the private worker flag, before any GUI initialization.
pub fn worker_entry() -> bool {
    let mut args = std::env::args_os();
    let _ = args.next();
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--vara-internal-file-worker")) {
        return false;
    }
    if let (Some(path), None) = (args.next(), args.next()) {
        let _ = serve(Path::new(&path));
    }
    true
}
