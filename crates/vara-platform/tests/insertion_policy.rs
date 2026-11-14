//! Compile and execute the same pure policy header used by macos.m. No AX/UI calls.
#[cfg(target_os = "macos")]
#[test]
fn native_insertion_policy_has_safe_pre_and_post_dispatch_decisions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable =
        std::env::temp_dir().join(format!("vara-insertion-policy-{}", std::process::id()));
    let status = std::process::Command::new("/usr/bin/clang")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("tests/insertion_policy.c"))
        .arg("-I")
        .arg(root.join("src"))
        .arg("-o")
        .arg(&executable)
        .status()
        .unwrap();
    assert!(status.success());
    let output = std::process::Command::new(&executable).output().unwrap();
    std::fs::remove_file(executable).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn native_wire_diagnostics_are_closed_and_capture_tokens_remain_runtime_only() {
    let capture: vara_core::Destination = serde_json::from_value(serde_json::json!({
        "app_id":"fixture.editor", "app_name":"Fixture", "target_token":"live-token",
        "capture_diagnostic": {"reason":"capture_failed", "method":null, "os_error":-25204}
    }))
    .unwrap();
    assert_eq!(capture.target_token.as_deref(), Some("live-token"));
    assert_eq!(capture.capture_diagnostic.unwrap().os_error, Some(-25204));
    assert!(
        serde_json::to_value(capture)
            .unwrap()
            .get("target_token")
            .is_none()
    );
    for (delivery, reason, method, error) in [
        (
            "inserted",
            "confirmed",
            "accessibility",
            serde_json::Value::Null,
        ),
        (
            "unknown",
            "handoff_unconfirmed",
            "accessibility",
            serde_json::json!(-25202),
        ),
        (
            "manual_ready",
            "post_event_access_missing",
            "keyboard",
            serde_json::Value::Null,
        ),
        (
            "unknown",
            "focus_changed",
            "keyboard",
            serde_json::Value::Null,
        ),
    ] {
        let wire = serde_json::json!({"delivery":delivery,"diagnostic":{"reason":reason,"method":method,"os_error":error}});
        let parsed: vara_core::InsertionOutcome = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), wire);
    }
    assert!(
        serde_json::from_value::<vara_core::InsertionDiagnostic>(serde_json::json!({
            "reason":"arbitrary error text", "method":null, "os_error":null
        }))
        .is_err()
    );
}
