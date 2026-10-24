use crate::{Cancellation, Result};
use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::Path,
};
#[derive(Clone, Copy)]
pub struct Artifact {
    pub name: &'static str,
    pub url: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
}
const PARAKEET: &[Artifact] = &[
    Artifact {
        name: "config.json",
        url: "https://huggingface.co/moondream/parakeet-redux/resolve/2bf128600aac4b16946f7ed8372e56117fe5e23b/config.json",
        bytes: 12988,
        sha256: "503c653b2e3bb788adbcb04f5abdee532d958686564081baeed133ff10143f6e",
    },
    Artifact {
        name: "model.safetensors",
        url: "https://huggingface.co/moondream/parakeet-redux/resolve/2bf128600aac4b16946f7ed8372e56117fe5e23b/model.safetensors",
        bytes: 177774490,
        sha256: "78ec25733ee0d0c1586d1346fc86db9d0c2e436e3a8ab1d32a82d1bb8f848d21",
    },
    Artifact {
        name: "ternary.json",
        url: "https://huggingface.co/moondream/parakeet-redux/resolve/2bf128600aac4b16946f7ed8372e56117fe5e23b/ternary.json",
        bytes: 57970,
        sha256: "1221c6d3ce901ffe09c089da758a8db8b76189f80cff41c5afc244fc61e2051d",
    },
    Artifact {
        name: "tokenizer.json",
        url: "https://huggingface.co/moondream/parakeet-redux/resolve/2bf128600aac4b16946f7ed8372e56117fe5e23b/tokenizer.json",
        bytes: 1159960,
        sha256: "bd321b096832a3f270bd3b2a88823957920f1a5c5ada71114a26ea729d0cbe91",
    },
];
const SPEECH: &[Artifact] = &[Artifact {
    name: "ggml-base.bin",
    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.bin",
    bytes: 147951465,
    sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
}];
const TEXT: &[Artifact] = &[
    Artifact {
        name: "Qwen3-0.6B-Q8_0.gguf",
        url: "https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/resolve/23749fefcc72300e3a2ad315e1317431b06b590a/Qwen3-0.6B-Q8_0.gguf",
        bytes: 639446688,
        sha256: "9465e63a22add5354d9bb4b99e90117043c7124007664907259bd16d043bb031",
    },
    Artifact {
        name: "tokenizer.json",
        url: "https://huggingface.co/Qwen/Qwen3-0.6B/resolve/c1899de289a04d12100db370d81485cdf75e47ca/tokenizer.json",
        bytes: 11422654,
        sha256: "aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4",
    },
];
pub fn artifacts(id: &str) -> Option<&'static [Artifact]> {
    match id {
        "local.parakeet" => Some(PARAKEET),
        "local.whisper" => Some(SPEECH),
        "local.text" => Some(TEXT),
        _ => None,
    }
}
pub fn model_directory(root: &Path, id: &str) -> std::path::PathBuf {
    if id == "local.parakeet" {
        root.join("parakeet-redux")
    } else {
        root.to_path_buf()
    }
}
#[derive(Clone, Serialize)]
pub struct Progress {
    pub id: String,
    pub received: u64,
    pub total: u64,
    pub phase: String,
}
pub async fn install(
    id: &str,
    dir: &Path,
    cancel: &Cancellation,
    progress: impl Fn(Progress),
) -> Result<()> {
    let files = artifacts(id).ok_or("Ukendt lokal model.")?;
    let directory = model_directory(dir, id);
    let dir = directory.as_path();
    std::fs::create_dir_all(dir).map_err(|_| "Modelmappen kunne ikke oprettes.")?;
    // Model hosts redirect to a CDN. Credentials are never attached to this download client.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1800))
        .build()
        .map_err(|_| "Download kunne ikke starte.")?;
    let total = files.iter().map(|f| f.bytes).sum();
    let mut received = 0;
    let mut pending = Vec::new();
    let outcome=async {
    for a in files {
        cancel.check()?;
        let dest = dir.join(a.name);
        if verified(&dest, a)? {
            received += a.bytes;
            continue;
        }
        let part = dir.join(format!("{}.part", a.name));
        let result = async {
            let response = client
                .get(a.url)
                .send()
                .await
                .map_err(|_| "Kunne ikke hente modellen. Kontroller forbindelsen.")?
                .error_for_status()
                .map_err(|_| "Modelserveren afviste download.")?;
            let mut file =
                std::fs::File::create(&part).map_err(|_| "Kunne ikke gemme modeldownload.")?;
            let mut stream = response.bytes_stream();
            let mut n = 0;
            let mut hash = Sha256::new();
            let mut last = std::time::Instant::now();
            let mut last_byte=std::time::Instant::now();
            loop {
                let chunk=tokio::select!{chunk=stream.next()=>chunk,_=tokio::time::sleep(std::time::Duration::from_millis(100))=>{cancel.check()?;if last_byte.elapsed()>std::time::Duration::from_secs(45){return Err("Download er gået i stå. Prøv igen.".into());}continue;}};
                let Some(chunk)=chunk else {break;};last_byte=std::time::Instant::now();cancel.check()?;
                let chunk = chunk.map_err(|_| "Download blev afbrudt. Prøv igen.")?;
                n += chunk.len() as u64;
                if n > a.bytes {
                    return Err("Download har forkert størrelse.".into());
                }
                hash.update(&chunk);
                file.write_all(&chunk)
                    .map_err(|_| "Ikke plads til modellen.")?;
                if last.elapsed() > std::time::Duration::from_millis(150) {
                    progress(Progress {
                        id: id.into(),
                        received: received + n,
                        total,
                        phase: "downloading".into(),
                    });
                    last = std::time::Instant::now();
                }
            }
            cancel.check()?;
            if n != a.bytes || format!("{:x}", hash.finalize()) != a.sha256 {
                return Err("Modelkontrol fejlede. Download er ikke taget i brug.".into());
            }
            file.sync_all()
                .map_err(|_| "Modellen kunne ikke gemmes sikkert.")?;
            cancel.check()?;

            Ok::<(), String>(())
        }
        .await;
        if result.is_err() {
            let _ = std::fs::remove_file(&part);
        }
        result?;
        pending.push((part,dest));
        received += a.bytes;
    }
    cancel.publish(||{let mut published=Vec::new();for (part,dest) in &pending{if std::fs::rename(part,dest).is_err(){for path in published{let _=std::fs::remove_file(path);}return Err("Kunne ikke installere modellen.".into());}published.push(dest);}Ok(())})?;
    progress(Progress {
        id: id.into(),
        received,
        total,
        phase: "ready".into(),
    });
    Ok(())
    }.await;
    if outcome.is_err() {
        for a in files {
            let _ = std::fs::remove_file(dir.join(format!("{}.part", a.name)));
        }
    }
    outcome
}
pub fn verified(path: &Path, a: &Artifact) -> Result<bool> {
    if !std::fs::metadata(path).is_ok_and(|m| m.len() == a.bytes) {
        return Ok(false);
    }
    let mut f = std::fs::File::open(path).map_err(|_| "Modellen kunne ikke læses.")?;
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|_| "Modellen kunne ikke kontrolleres.")?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()) == a.sha256)
}
