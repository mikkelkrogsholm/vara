//! Agent interface to the same durable local file service as the desktop.
use crate::local_backend::{self, Operation, RpcError};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use vara_core::{FileStage, TranscriptionDocument, UserMessage};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn interrupt(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

const HELP: &str = "Vara agent CLI v1\n\n  cli transcribe FILE [--model ID] [--language CODE] [--request-id ID] [--no-wait] [--timeout SECONDS] [--format json|txt|md|srt|vtt] [--output FILE] [--overwrite]\n  cli job status|wait|result|cancel ID [--timeout SECONDS] [--format FORMAT] [--output FILE] [--overwrite]\n  cli models [--json]\n  cli --help | --version\n\nDefault model: configured speech engine. Default language: automatic. Redux detects language automatically. Default is to wait without a deadline. --no-wait returns a durable job reference; --output cannot be combined with it (use job result later). Each submit prints its request ID before IPC and job ID after acceptance; retry identical input with --request-id after a lost receipt. Timeout or Ctrl-C stops waiting, not the job; use job cancel to cancel. JSON is the default. An explicit non-JSON format without --output writes only export content; --output returns a JSON receipt. Existing output requires --overwrite. No model is downloaded.\nExit: 0 success, 2 usage/input, 3 unavailable/privacy, 4 failed/cancelled job, 5 backend, 6 export, 7 interrupted/timeout.\n";
#[derive(Default)]
struct Options {
    model: Option<String>,
    language: Option<String>,
    request_id: Option<String>,
    no_wait: bool,
    timeout: Option<Duration>,
    format: Option<String>,
    output: Option<PathBuf>,
    overwrite: bool,
}
struct Failure {
    code: String,
    message: Option<UserMessage>,
    exit: i32,
    job_id: Option<String>,
    request_id: Option<String>,
    status: Option<String>,
}
impl Failure {
    fn new(code: &str, exit: i32) -> Self {
        Self {
            code: code.into(),
            message: None,
            exit,
            job_id: None,
            request_id: None,
            status: None,
        }
    }
    fn rpc(e: RpcError) -> Self {
        let code = e
            .message
            .as_ref()
            .map(|m| m.code.as_str())
            .unwrap_or(&e.code);
        let exit = classify(code);
        Self {
            code: code.to_owned(),
            message: e.message,
            exit,
            job_id: None,
            request_id: None,
            status: None,
        }
    }
}
fn classify(code: &str) -> i32 {
    match code {
        "file.source_missing"
        | "file.choose_first"
        | "file.choose_count"
        | "file.id_invalid"
        | "file.request_invalid"
        | "file.request_conflict"
        | "file.language_unsupported" => 2,
        "file.choose_speech"
        | "file.model_download"
        | "file.privacy_only"
        | "provider.missing_key" => 3,
        "file.runtime_unavailable"
        | "local.model_missing"
        | "local.model_corrupt"
        | "local.model_load"
        | "parakeet.model_missing"
        | "engine.speech_unsupported" => 3,
        s if s.starts_with("keyring.") => 3,
        s if s.starts_with("backend.") => 5,
        _ => 4,
    }
}
fn fail(f: Failure) -> i32 {
    let _ = writeln!(
        io::stdout().lock(),
        "{}",
        json!({"schema_version":1,"ok":false,"job_id":f.job_id,"document_id":f.job_id,"request_id":f.request_id,"status":f.status,"error":{"code":f.code,"message":f.message}})
    );
    f.exit
}
fn emit(v: Value) -> Result<(), Failure> {
    writeln!(io::stdout().lock(), "{v}").map_err(|_| Failure::new("cli.output_closed", 7))
}
fn pipe_closed() -> bool {
    #[cfg(unix)]
    {
        let mut fd = libc::pollfd {
            fd: libc::STDOUT_FILENO,
            events: libc::POLLOUT,
            revents: 0,
        };
        unsafe { libc::poll(&mut fd, 1, 0) };
        return fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0;
    }
    #[cfg(not(unix))]
    {
        false
    }
}
fn check(deadline: Option<Instant>) -> Result<(), Failure> {
    if INTERRUPTED.load(Ordering::SeqCst) {
        return Err(Failure::new("cli.interrupted", 7));
    }
    if pipe_closed() {
        return Err(Failure::new("cli.output_closed", 7));
    }
    if deadline.is_some_and(|d| Instant::now() >= d) {
        return Err(Failure::new("cli.timeout", 7));
    }
    Ok(())
}
fn call(
    root: &Path,
    op: Operation,
    exe: &Path,
    deadline: Option<Instant>,
) -> Result<Value, Failure> {
    check(deadline)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let root = root.to_owned();
    let exe = exe.to_owned();
    std::thread::spawn(move || {
        let _ = tx.send(local_backend::call(&root, op, &exe));
    });
    loop {
        check(deadline)?;
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(r)) => {
                return match r.error {
                    Some(e) => Err(Failure::rpc(e)),
                    None => Ok(r.result.unwrap_or(Value::Null)),
                };
            }
            Ok(Err(e)) => return Err(Failure::new(&e, 5)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err(Failure::new("backend.transport", 5)),
        }
    }
}
fn parse_options(args: &[String], allowed: &[&str]) -> Result<Options, Failure> {
    let mut o = Options::default();
    let mut seen = std::collections::HashSet::new();
    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        if !allowed.contains(&key) || !seen.insert(key) {
            return Err(Failure::new("cli.invalid_arguments", 2));
        }
        i += 1;
        match key {
            "--no-wait" => o.no_wait = true,
            "--overwrite" => o.overwrite = true,
            "--json" => {}
            _ => {
                let value = args
                    .get(i)
                    .filter(|s| !s.is_empty() && !s.starts_with("--"))
                    .ok_or_else(|| Failure::new("cli.invalid_arguments", 2))?;
                i += 1;
                match key {
                    "--model" => o.model = Some(value.clone()),
                    "--language" => o.language = Some(value.clone()),
                    "--request-id" => o.request_id = Some(value.clone()),
                    "--format" => o.format = Some(value.clone()),
                    "--output" => o.output = Some(PathBuf::from(value)),
                    "--timeout" => {
                        let n = value
                            .parse::<f64>()
                            .map_err(|_| Failure::new("cli.invalid_arguments", 2))?;
                        if !n.is_finite() || n <= 0. || n > 31_536_000. {
                            return Err(Failure::new("cli.invalid_arguments", 2));
                        }
                        o.timeout = Some(Duration::from_secs_f64(n));
                    }
                    _ => return Err(Failure::new("cli.invalid_arguments", 2)),
                }
            }
        }
    }
    if o.format
        .as_deref()
        .is_some_and(|f| !["json", "txt", "md", "srt", "vtt"].contains(&f))
        || o.overwrite && o.output.is_none()
    {
        return Err(Failure::new("cli.invalid_arguments", 2));
    }
    Ok(o)
}
fn document(v: Value) -> Result<TranscriptionDocument, Failure> {
    serde_json::from_value(v).map_err(|_| Failure::new("backend.response_invalid", 5))
}
fn receipt(d: &TranscriptionDocument) -> Value {
    json!({"schema_version":1,"ok":true,"job_id":d.id,"document_id":d.id,"status":d.stage,"timing_capability":d.engine.file_timing_capability()})
}
fn terminal(d: &TranscriptionDocument) -> Result<bool, Failure> {
    match d.stage {
        FileStage::Ready => Ok(true),
        FileStage::Failed | FileStage::Cancelled => {
            let message = d.error.as_deref().and_then(UserMessage::decode);
            let code = message.as_ref().map(|m| m.code.clone()).unwrap_or_else(|| {
                if d.stage == FileStage::Cancelled {
                    "file.job_cancelled".into()
                } else {
                    "file.job_failed".into()
                }
            });
            let exit = classify(&code);
            Err(Failure {
                code,
                message,
                exit,
                job_id: Some(d.id.clone()),
                request_id: None,
                status: Some(
                    if d.stage == FileStage::Cancelled {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into(),
                ),
            })
        }
        _ => Ok(false),
    }
}
fn wait_inner(
    root: &Path,
    exe: &Path,
    id: &str,
    deadline: Option<Instant>,
) -> Result<TranscriptionDocument, Failure> {
    loop {
        let d = document(call(
            root,
            Operation::Status { id: id.into() },
            exe,
            deadline,
        )?)?;
        if terminal(&d)? {
            return Ok(d);
        };
        check(deadline)?;
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait(
    root: &Path,
    exe: &Path,
    id: &str,
    deadline: Option<Instant>,
) -> Result<TranscriptionDocument, Failure> {
    wait_inner(root, exe, id, deadline).map_err(|mut e| {
        e.job_id = Some(id.into());
        e
    })
}

fn output(
    root: &Path,
    exe: &Path,
    d: TranscriptionDocument,
    o: &Options,
    deadline: Option<Instant>,
) -> Result<(), Failure> {
    let mut r = receipt(&d);
    let format = o.format.as_deref().unwrap_or("json");
    let data = d.export(format, false).map_err(|e| {
        let message = UserMessage::decode(&e.wire());
        Failure {
            code: message
                .as_ref()
                .map(|m| m.code.clone())
                .unwrap_or_else(|| "cli.export_failed".into()),
            message,
            exit: 6,
            job_id: Some(d.id.clone()),
            request_id: None,
            status: Some("ready".into()),
        }
    })?;
    if o.output.is_some() || format != "json" {
        if let Some(path) = &o.output {
            let absolute = std::path::absolute(path).map_err(|_| {
                let mut e = Failure::new("file.export_save", 6);
                e.job_id = Some(d.id.clone());
                e.status = Some("ready".into());
                e
            })?;
            call(
                root,
                Operation::Export {
                    id: d.id.clone(),
                    format: format.into(),
                    path: absolute,
                    overwrite: o.overwrite,
                },
                exe,
                deadline,
            )
            .map_err(|mut e| {
                e.job_id = Some(d.id.clone());
                e.status = Some("ready".into());
                if e.exit != 7 && e.exit != 5 {
                    e.exit = 6;
                }
                e
            })?;
            r["output"] = json!({"path":path,"format":format});
            emit(r)
        } else {
            io::stdout()
                .lock()
                .write_all(data.as_bytes())
                .map_err(|_| Failure::new("cli.output_closed", 7))
        }
    } else {
        r["document"] =
            serde_json::from_str(&data).map_err(|_| Failure::new("backend.response_invalid", 5))?;
        emit(r)
    }
}
fn run(args: Vec<String>) -> Result<(), Failure> {
    if args.len() == 1 && args[0] == "--help" {
        print!("{HELP}");
        return Ok(());
    }
    if args.len() == 1 && args[0] == "--version" {
        println!("vara {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let root = local_backend::default_profile().map_err(|e| Failure::new(&e, 5))?;
    let exe = std::env::current_exe().map_err(|_| Failure::new("backend.start_failed", 5))?;
    match args.first().map(String::as_str) {
        Some("models") => {
            parse_options(&args[1..], &["--json"])?;
            let v = call(&root, Operation::Models, &exe, None)?;
            let mut r = json!({"schema_version":1,"ok":true});
            if let Value::Object(m) = v {
                for (k, v) in m {
                    r[k] = v
                }
            }
            emit(r)
        }
        Some("transcribe") => {
            let source = args
                .get(1)
                .filter(|s| !s.starts_with('-'))
                .ok_or_else(|| Failure::new("cli.invalid_arguments", 2))?;
            let o = parse_options(
                &args[2..],
                &[
                    "--model",
                    "--language",
                    "--request-id",
                    "--no-wait",
                    "--timeout",
                    "--format",
                    "--output",
                    "--overwrite",
                ],
            )?;
            if o.no_wait && o.output.is_some() {
                return Err(Failure::new("cli.invalid_arguments", 2));
            }
            let source =
                std::path::absolute(source).map_err(|_| Failure::new("file.source_missing", 2))?;
            // Resolve the parent even if a previously submitted source was removed.
            let source = match (source.parent(), source.file_name()) {
                (Some(parent), Some(name)) => std::fs::canonicalize(parent)
                    .map(|p| p.join(name))
                    .unwrap_or(source.clone()),
                _ => source,
            };
            let request_id = o
                .request_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let _ = writeln!(io::stderr().lock(), "request_id: {request_id}");
            let deadline = o.timeout.map(|t| Instant::now() + t);
            let r = call(
                &root,
                Operation::Submit {
                    request: vara_app::SubmitRequest {
                        source,
                        engine_id: o.model.clone(),
                        language: o.language.clone(),
                        request_id: Some(request_id.clone()),
                    },
                },
                &exe,
                deadline,
            )
            .map_err(|mut e| {
                e.request_id = Some(request_id.clone());
                e
            })?;
            let id = r["job_id"]
                .as_str()
                .ok_or_else(|| Failure::new("backend.response_invalid", 5))?;
            let _ = writeln!(io::stderr().lock(), "job_id: {id}");
            if o.no_wait {
                emit(
                    json!({"schema_version":1,"ok":true,"job_id":id,"document_id":id,"status":"accepted","request_id":request_id}),
                )
            } else {
                output(&root, &exe, wait(&root, &exe, id, deadline)?, &o, deadline)
            }
        }
        Some("job") => {
            let action = args
                .get(1)
                .map(String::as_str)
                .ok_or_else(|| Failure::new("cli.invalid_arguments", 2))?;
            let id = args
                .get(2)
                .filter(|s| !s.starts_with('-'))
                .ok_or_else(|| Failure::new("cli.invalid_arguments", 2))?;
            let allowed = match action {
                "status" | "cancel" => &[][..],
                "wait" => &["--timeout", "--format", "--output", "--overwrite"][..],
                "result" => &["--format", "--output", "--overwrite"][..],
                _ => return Err(Failure::new("cli.invalid_arguments", 2)),
            };
            let o = parse_options(&args[3..], allowed)?;
            let deadline = o.timeout.map(|t| Instant::now() + t);
            match action {
                "status" => {
                    let d = document(call(
                        &root,
                        Operation::Status { id: id.clone() },
                        &exe,
                        deadline,
                    )?)?;
                    terminal(&d)?;
                    emit(receipt(&d))
                }
                "cancel" => {
                    call(&root, Operation::Cancel { id: id.clone() }, &exe, deadline)?;
                    let d = document(call(
                        &root,
                        Operation::Status { id: id.clone() },
                        &exe,
                        deadline,
                    )?)?;
                    emit(receipt(&d))
                }
                "wait" => output(&root, &exe, wait(&root, &exe, id, deadline)?, &o, deadline),
                "result" => {
                    let d = document(call(
                        &root,
                        Operation::Status { id: id.clone() },
                        &exe,
                        deadline,
                    )?)?;
                    if terminal(&d)? {
                        output(&root, &exe, d, &o, deadline)
                    } else {
                        emit(receipt(&d))
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => Err(Failure::new("cli.invalid_arguments", 2)),
    }
}
pub fn entry() -> i32 {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, interrupt as *const () as libc::sighandler_t);
    }
    let args = std::env::args_os()
        .skip(2)
        .map(|arg| arg.into_string())
        .collect::<Result<Vec<_>, _>>();
    match args {
        Ok(args) => match run(args) {
            Ok(()) => 0,
            Err(e) => fail(e),
        },
        Err(_) => fail(Failure::new("cli.invalid_arguments", 2)),
    }
}
