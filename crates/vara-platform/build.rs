use std::{env, fs, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=src/macos_text_input.h");
    for file in [
        "src/macos.m",
        "src/macos_hud.m",
        "src/macos_shortcut.m",
        "src/macos_credentials.m",
    ] {
        println!("cargo:rerun-if-changed={file}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("../../gdd/design/styles/tokens.css");
        println!("cargo:rerun-if-changed={}", source.display());
        let tokens = fs::read_to_string(source).expect("canonical design tokens");
        let mut header = String::from("// Generated from DESIGN.md tokens; do not edit.\n");
        for token in [
            "surface",
            "surface-muted",
            "primary",
            "on-surface",
            "muted",
            "divider",
        ] {
            let prefix = format!("--color-{token}:");
            let value = tokens
                .lines()
                .find_map(|l| l.trim().strip_prefix(&prefix))
                .expect("design color")
                .trim()
                .trim_end_matches(';')
                .trim_start_matches('#');
            let rgb = u32::from_str_radix(value, 16).expect("hex design color");
            header.push_str(&format!("#define VARA_COLOR_{} [NSColor colorWithRed:({}/255.0) green:({}/255.0) blue:({}/255.0) alpha:1]\n",token.to_uppercase().replace('-',"_"),(rgb>>16)&255,(rgb>>8)&255,rgb&255));
        }
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        fs::write(out.join("vara_design.h"), header).expect("native design header");
        cc::Build::new()
            .files([
                "src/macos.m",
                "src/macos_hud.m",
                "src/macos_shortcut.m",
                "src/macos_credentials.m",
            ])
            .include(out)
            .flag("-fobjc-arc")
            .compile("vara_native");
        for framework in [
            "AppKit",
            "ApplicationServices",
            "AVFoundation",
            "CoreAudio",
            "QuartzCore",
            "Security",
        ] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
