use std::{env, fs, path::Path};
fn copy(source: &Path, dest: &Path) {
    println!("cargo:rerun-if-changed={}", source.display());
    if source.is_dir() {
        fs::create_dir_all(dest).expect("design directory");
        for entry in fs::read_dir(source).expect("design assets") {
            let entry = entry.expect("design asset");
            copy(&entry.path(), &dest.join(entry.file_name()));
        }
    } else if fs::read(dest).ok() != Some(fs::read(source).expect("canonical design source")) {
        fs::create_dir_all(dest.parent().unwrap()).expect("design directory");
        fs::copy(source, dest).expect("package canonical design");
    }
}
fn main() {
    let manifest = std::path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.join("../..");
    let source = root.join("gdd/design");
    let ui = root.join("desktop/ui");
    for file in ["styles/tokens.css", "styles/app.css", "js/components.js"] {
        copy(&source.join(file), &ui.join("design").join(file));
    }
    copy(&source.join("assets"), &ui.join("assets"));
    println!("cargo:rerun-if-changed={}", ui.display());
    tauri_build::build();
}
