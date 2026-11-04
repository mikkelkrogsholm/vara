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
    // Optional runtime is produced by the platform packaging script. A source
    // checkout without it still builds; Who reports unavailable explicitly.
    fs::create_dir_all(manifest.join("runtime/who")).expect("Who resource directory");
    if env::var_os("CARGO_FEATURE_TEST_HARNESS").is_some()
        && env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
    {
        println!("cargo:rerun-if-changed=../tests/dialog-native.m");
        cc::Build::new()
            .file(manifest.join("../tests/dialog-native.m"))
            .flag("-fobjc-arc")
            .compile("vara_dialog_fixture");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
    tauri_build::build();
}
