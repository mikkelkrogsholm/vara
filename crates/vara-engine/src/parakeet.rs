use crate::{Cancellation, Result};
use parking_lot::Mutex;
use std::path::Path;
use voxrt::{
    longform::LongForm,
    ternary_linear::Precision,
    transcriber::{Device, Transcriber},
};

#[derive(Default)]
pub struct Parakeet {
    runtime: Mutex<Option<(Transcriber, rayon::ThreadPool)>>,
}
impl Parakeet {
    pub fn speech(&self, dir: &Path, audio: &Path, cancel: &Cancellation) -> Result<String> {
        // A backend panic must become an ordinary failed recognition, retaining retry audio.
        // The host's durable generation gate still owns cancellation/save/delivery.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.recognize(dir, audio, cancel)
        })) {
            Ok(result) => result,
            Err(_) => {
                *self.runtime.lock() = None;
                Err("Den lokale talemotor blev afbrudt. Prøv optagelsen igen.".into())
            }
        }
    }
    fn recognize(&self, dir: &Path, audio: &Path, cancel: &Cancellation) -> Result<String> {
        cancel.check()?;
        let (samples, _) = crate::local::read_audio(audio)?;
        if samples.len() < 1600 {
            return Err("Optagelsen er for kort.".into());
        }
        let mut guard = self.runtime.lock();
        cancel.check()?;
        if guard.is_none() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .map_err(|_| "Talemotorens arbejdstråde kunne ikke starte.")?;
            // Explicit backend selection ignores external VOXRT_DEVICE overrides.
            // Auto uses Metal on supported Apple builds and CPU elsewhere.
            let model = pool
                .install(|| Transcriber::load_on(dir, Precision::Int8, Device::Auto))
                .map_err(
                    |_| "Parakeet Redux kunne ikke indlæses. Hent modellen igen i Forbindelser.",
                )?;
            *guard = Some((model, pool));
        }
        cancel.check()?;
        let (model, pool) = guard.as_ref().unwrap();
        pool.install(|| {
            let mut stream = LongForm::new(model);
            for chunk in samples.chunks(10 * 16000) {
                cancel.check()?;
                stream.push(chunk);
            }
            cancel.check()?;
            stream.flush();
            cancel.check()?;
            Ok(stream.text())
        })
    }
}
