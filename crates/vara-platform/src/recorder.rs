use crate::Result;
use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc,
    },
};
#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputDevice {
    pub id: String,
    pub label: String,
}
#[cfg(target_os = "macos")]
pub fn input_devices() -> Result<Vec<InputDevice>> {
    unsafe extern "C" {
        fn vara_input_devices() -> *mut std::ffi::c_char;
    }
    unsafe {
        let raw = vara_input_devices();
        if raw.is_null() {
            return Err("Mikrofoner kunne ikke findes.".into());
        }
        let result = serde_json::from_slice(std::ffi::CStr::from_ptr(raw).to_bytes())
            .map_err(|_| "Mikrofonlisten kunne ikke læses.".into());
        crate::vara_free(raw);
        result
    }
}
#[cfg(not(target_os = "macos"))]
pub fn input_devices() -> Result<Vec<InputDevice>> {
    cpal::default_host()
        .input_devices()
        .map_err(|_| "Mikrofoner kunne ikke findes.")?
        .map(|d| {
            d.name()
                .map(|name| InputDevice {
                    id: name.clone(),
                    label: name,
                })
                .map_err(|_| "En mikrofon kunne ikke læses.".into())
        })
        .collect()
}
enum Command {
    Start(PathBuf, Option<String>, mpsc::Sender<Result<()>>),
    Stop(bool, mpsc::Sender<Result<f64>>),
}
#[derive(Clone)]
pub struct Recorder {
    tx: mpsc::Sender<Command>,
    level: Arc<AtomicU32>,
}
struct Capture {
    samples: Vec<i16>,
    error: Option<String>,
    rate: u32,
    level: Arc<AtomicU32>,
}
impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}
impl Recorder {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let level = Arc::new(AtomicU32::new(0));
        let capture_level = level.clone();
        std::thread::spawn(move || {
            let mut live: Option<(cpal::Stream, Arc<Mutex<Capture>>, PathBuf)> = None;
            while let Ok(command) = rx.recv() {
                match command {
                    Command::Start(path, device, reply) => {
                        let result = if live.is_some() {
                            Err("Der optages allerede.".into())
                        } else {
                            start(device, capture_level.clone()).map(|(stream, capture)| {
                                live = Some((stream, capture, path));
                            })
                        };
                        let _ = reply.send(result);
                    }
                    Command::Stop(save, reply) => {
                        let result = if let Some((stream, capture, path)) = live.take() {
                            drop(stream);
                            let c = capture.lock();
                            if !save {
                                let _ = std::fs::remove_file(path);
                                Ok(0.)
                            } else if let Some(error) = &c.error {
                                Err(error.clone())
                            } else {
                                write(&path, &c)
                            }
                        } else {
                            Err("Der er ingen optagelse i gang.".into())
                        };
                        let _ = reply.send(result);
                    }
                }
            }
        });
        Self { tx, level }
    }
    pub fn start(&self, path: PathBuf, device: Option<String>) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Start(path, device, tx))
            .map_err(|_| "Lydmotoren er stoppet.")?;
        rx.recv().map_err(|_| "Lydmotoren svarede ikke.")?
    }
    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
    pub fn stop(&self, save: bool) -> Result<f64> {
        self.level.store(0, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Stop(save, tx))
            .map_err(|_| "Lydmotoren er stoppet.")?;
        rx.recv().map_err(|_| "Lydmotoren svarede ikke.")?
    }
}
fn append<T: cpal::Sample>(data: &[T], channels: usize, capture: &Arc<Mutex<Capture>>)
where
    i16: cpal::FromSample<T>,
{
    let mut c = capture.lock();
    if c.error.is_some() {
        return;
    }
    if c.samples.len() + data.len() / channels > c.rate as usize * 600 {
        c.error =
            Some("Optagelsen nåede grænsen på 10 minutter. Start en kortere diktering.".into());
        return;
    }
    let start = c.samples.len();
    c.samples.extend(data.chunks(channels).map(|frame| {
        let sum: i64 = frame.iter().map(|s| i16::from_sample(*s) as i64).sum();
        (sum / frame.len() as i64) as i16
    }));
    let samples = &c.samples[start..];
    let rms = (samples
        .iter()
        .map(|x| (*x as f32 / 32768.).powi(2))
        .sum::<f32>()
        / samples.len().max(1) as f32)
        .sqrt();
    let normalized = ((20. * rms.max(0.00001).log10() + 60.) / 60.).clamp(0., 1.);
    c.level.store(normalized.to_bits(), Ordering::Relaxed);
}
fn start(
    name: Option<String>,
    level: Arc<AtomicU32>,
) -> Result<(cpal::Stream, Arc<Mutex<Capture>>)> {
    let host = cpal::default_host();
    let device = if let Some(name) = name {
        // Do not probe every device's supported formats just to find a name.
        host.devices()
            .map_err(|_| "Mikrofonerne kunne ikke læses.")?
            .find(|d| d.name().is_ok_and(|n| n == name))
    } else {
        host.default_input_device()
    }
    .ok_or("Den valgte mikrofon er ikke tilgængelig. Vælg en anden i Indstillinger.")?;
    let supported = device
        .default_input_config()
        .map_err(|_| "Mikrofonens lydformat er ikke tilgængeligt.")?;
    let config: cpal::StreamConfig = supported.clone().into();
    let channels = config.channels as usize;
    let capture = Arc::new(Mutex::new(Capture {
        samples: Vec::new(),
        error: None,
        rate: config.sample_rate.0,
        level,
    }));
    let data = capture.clone();
    let errors = capture.clone();
    let on_error = move |_: cpal::StreamError| {
        errors.lock().error =
            Some("Mikrofonen blev afbrudt. Kontroller forbindelsen og prøv igen.".into());
    };
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |v: &[f32], _| append(v, channels, &data),
            on_error,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |v: &[i16], _| append(v, channels, &data),
            on_error,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |v: &[u16], _| append(v, channels, &data),
            on_error,
            None,
        ),
        _ => return Err("Mikrofonens sampleformat understøttes ikke endnu.".into()),
    }
    .map_err(|_| "Mikrofonen kunne ikke åbnes. Kontroller tilladelsen i systemindstillingerne.")?;
    stream.play().map_err(|_| "Mikrofonen kunne ikke starte.")?;
    Ok((stream, capture))
}
fn write(path: &std::path::Path, c: &Capture) -> Result<f64> {
    if c.samples.is_empty() {
        return Err("Mikrofonen leverede ingen lyd.".into());
    }
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: c.rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .map_err(|_| "Optagelsen kunne ikke gemmes.")?;
    for sample in &c.samples {
        writer
            .write_sample(*sample)
            .map_err(|_| "Der er ikke plads til optagelsen.")?;
    }
    writer
        .finalize()
        .map_err(|_| "Optagelsen kunne ikke afsluttes.")?;
    Ok(c.samples.len() as f64 / c.rate as f64)
}
