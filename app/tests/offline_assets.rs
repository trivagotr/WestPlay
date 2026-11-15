//! A packaged companion must start without the Vite development server.
#![cfg(feature = "custom-protocol")]
#[test]
fn packaged_context_contains_the_companion_and_its_script() {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let index = context.assets().get(&"index.html".into()).expect("packaged companion HTML must be embedded");
    let html = std::str::from_utf8(&index).unwrap();
    let script = html.split("src=\"").nth(1).unwrap().split('"').next().unwrap().trim_start_matches('/');
    assert!(context.assets().get(&script.into()).is_some(), "the companion's bundled script must also be embedded");
    assert!(context.assets().get(&"cabin.html".into()).is_some());
}
