//! A local rebuild may keep its version string but change the plugin/bridge ABI.
pub(super) fn needed(release: bool, current: &str, installed: Option<&str>, plugin_current: bool) -> bool {
    release && (installed != Some(current) || !plugin_current)
}
#[cfg(test)]
mod tests {
    use super::needed;
    #[test]
    fn same_version_with_different_plugin_still_requires_repair() {
        assert!(needed(true, "2.4.0", Some("2.4.0"), false));
    }
    #[test]
    fn matching_files_and_version_do_not_repeat_repair() {
        assert!(!needed(true, "2.4.0", Some("2.4.0"), true));
    }
    #[test]
    fn development_builds_never_replace_plugins() {
        assert!(!needed(false, "2.4.0", Some("2.3.0"), false));
    }
    #[test]
    fn older_or_missing_install_version_requires_update() {
        assert!(needed(true, "2.4.0", Some("2.3.0"), true));
        assert!(needed(true, "2.4.0", None, true));
    }
}
