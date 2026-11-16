use westplay_remote::ShellStyle;
#[test]
fn only_safe_cabin_colors_are_accepted() {
    let good: ShellStyle =
        serde_json::from_str(r##"{"appearance":"light","palette":["#ffeecc","#225588","#112233"]}"##).unwrap();
    assert!(good.valid());
    let bad: ShellStyle =
        serde_json::from_str(r##"{"appearance":"dark","palette":["url(https://example.com)","#225588","#112233"]}"##)
            .unwrap();
    assert!(!bad.valid());
    assert!(
        serde_json::from_str::<ShellStyle>(r##"{"appearance":"other","palette":["#112233","#112233","#112233"]}"##)
            .is_err()
    );
}
