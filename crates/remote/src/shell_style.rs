use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShellStyle {
    pub appearance: Appearance,
    pub palette: [String; 3],
}
impl Default for ShellStyle {
    fn default() -> Self {
        Self { appearance: Appearance::Dark, palette: ["#15151b".into(), "#25262e".into(), "#16181f".into()] }
    }
}
impl ShellStyle {
    pub fn valid(&self) -> bool {
        self.palette
            .iter()
            .all(|c| c.len() == 7 && c.starts_with('#') && c.as_bytes()[1..].iter().all(|b| b.is_ascii_hexdigit()))
    }
}
