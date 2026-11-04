use vara_core::{Settings, Store, UiLocale, resolve_ui_locale};

fn langs(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}

#[test]
fn resolves_preference_and_ordered_system_languages() {
    assert_eq!(
        resolve_ui_locale(UiLocale::System, &langs(&["da-DK"])),
        "da"
    );
    assert_eq!(
        resolve_ui_locale(UiLocale::System, &langs(&["en-GB"])),
        "en"
    );
    assert_eq!(
        resolve_ui_locale(UiLocale::System, &langs(&["fr-FR", "DA-dk", "en-US"])),
        "da"
    );
    assert_eq!(
        resolve_ui_locale(UiLocale::System, &langs(&["fr", "de"])),
        "en"
    );
    assert_eq!(resolve_ui_locale(UiLocale::System, &[]), "en");
    assert_eq!(resolve_ui_locale(UiLocale::Da, &langs(&["en"])), "da");
    assert_eq!(resolve_ui_locale(UiLocale::En, &langs(&["da"])), "en");
}

#[test]
fn fresh_legacy_invalid_and_persisted_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.sqlite3");
    let store = Store::open(&path).unwrap();
    assert_eq!(store.settings().unwrap().ui_locale, UiLocale::System);
    let mut old = serde_json::to_value(Settings::default()).unwrap();
    old.as_object_mut().unwrap().remove("ui_locale");
    old["language"] = serde_json::json!("sv");
    old["general_prompt"] = serde_json::json!("Personal instructions");
    old["strict_local"] = serde_json::json!(true);
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "INSERT INTO preferences (id, value) VALUES (1, ?1)",
        [old.to_string()],
    )
    .unwrap();
    drop(db);
    let mut loaded = store.settings().unwrap();
    assert_eq!(loaded.ui_locale, UiLocale::Da);
    assert_eq!(loaded.language.as_deref(), Some("sv"));
    assert_eq!(loaded.general_prompt, "Personal instructions");
    assert!(loaded.strict_local);
    loaded.ui_locale = UiLocale::En;
    store.save_settings(&loaded).unwrap();
    drop(store);
    let reopened = Store::open(&path).unwrap().settings().unwrap();
    assert_eq!(reopened.ui_locale, UiLocale::En);
    assert_eq!(reopened.language.as_deref(), Some("sv"));
    for preference in [UiLocale::System, UiLocale::Da, UiLocale::En] {
        let store = Store::open(&path).unwrap();
        let mut settings = store.settings().unwrap();
        settings.ui_locale = preference;
        store.save_settings(&settings).unwrap();
        drop(store);
        assert_eq!(
            Store::open(&path).unwrap().settings().unwrap().ui_locale,
            preference
        );
    }
    for invalid in [
        serde_json::json!("xx"),
        serde_json::json!(null),
        serde_json::json!(42),
    ] {
        old["ui_locale"] = invalid;
        assert_eq!(
            serde_json::from_value::<Settings>(old.clone())
                .unwrap()
                .ui_locale,
            UiLocale::En
        );
    }
}
