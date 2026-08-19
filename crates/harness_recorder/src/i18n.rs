//! Internationalization (i18n) module for harness_recorder.
//!
//! Loads JSON locale files from `locales/{lang}/messages.json` and provides
//! dot-notation key resolution with English fallback.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// I18n provides internationalization support by loading and resolving locale messages.
pub struct I18n {
    /// Currently active language code (e.g. "en", "fr", "de").
    lang: String,
    /// Loaded messages for the active language.
    messages: HashMap<String, String>,
    /// Fallback messages (English).
    fallback: HashMap<String, String>,
    /// Base path to the locales directory.
    locales_dir: PathBuf,
}

impl I18n {
    /// Create a new I18n instance for the given language.
    ///
    /// Loads `locales/{lang}/messages.json` and falls back to `locales/en/messages.json`
    /// for missing keys.
    pub fn new(lang: &str) -> Self {
        let locales_dir = Self::default_locales_dir();
        Self::with_dir(lang, &locales_dir)
    }

    /// Create a new I18n instance with a custom locales directory.
    pub fn with_dir(lang: &str, locales_dir: &Path) -> Self {
        let messages = Self::load_messages(locales_dir, lang);
        let fallback = if lang != "en" {
            Self::load_messages(locales_dir, "en")
        } else {
            HashMap::new()
        };

        Self {
            lang: lang.to_string(),
            messages,
            fallback,
            locales_dir: locales_dir.to_path_buf(),
        }
    }

    /// Translate a key using dot-notation (e.g. `"errors.file_not_found"`).
    ///
    /// If the key is not found in the active language, the English fallback is tried.
    /// Returns the raw key if not found anywhere.
    pub fn t(&self, key: &str) -> String {
        if let Some(val) = self.messages.get(key) {
            return val.clone();
        }
        if let Some(val) = self.fallback.get(key) {
            return val.clone();
        }
        key.to_string()
    }

    /// Translate a key with parameter interpolation.
    ///
    /// Placeholders in the message use `{param}` syntax. The `params` slice supplies
    /// values in order matching the placeholder names extracted after the key.
    pub fn t_with(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut result = self.t(key);
        for (name, value) in params {
            result = result.replace(&format!("{{{}}}", name), value);
        }
        result
    }

    /// Return the active language code.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    /// Return the default locales directory (`locales/` relative to CARGO_MANIFEST_DIR,
    /// or the current directory).
    fn default_locales_dir() -> PathBuf {
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest).join("locales")
        } else {
            PathBuf::from("locales")
        }
    }

    /// Load messages from `locales_dir/{lang}/messages.json`.
    fn load_messages(locales_dir: &Path, lang: &str) -> HashMap<String, String> {
        let path = locales_dir.join(lang).join("messages.json");
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return HashMap::new(),
        };

        let raw: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => return HashMap::new(),
        };

        let mut map = HashMap::new();
        Self::flatten_json(&raw, "", &mut map);
        map
    }

    /// Flatten a nested JSON object into dot-notation keys.
    fn flatten_json(value: &serde_json::Value, prefix: &str, out: &mut HashMap<String, String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    Self::flatten_json(v, &key, out);
                }
            }
            serde_json::Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_locales(dir: &Path) {
        // English
        let en_dir = dir.join("en");
        fs::create_dir_all(&en_dir).unwrap();
        fs::write(
            en_dir.join("messages.json"),
            r#"{
                "welcome": "Welcome to Helios CLI",
                "errors": {
                    "file_not_found": "File not found: {path}",
                    "permission_denied": "Permission denied: {path}"
                },
                "cli": {
                    "version": "Version {version}"
                }
            }"#,
        )
        .unwrap();

        // French
        let fr_dir = dir.join("fr");
        fs::create_dir_all(&fr_dir).unwrap();
        fs::write(
            fr_dir.join("messages.json"),
            r#"{
                "welcome": "Bienvenue dans Helios CLI",
                "errors": {
                    "file_not_found": "Fichier non trouvé : {path}"
                }
            }"#,
        )
        .unwrap();
    }

    #[test]
    fn test_t_english() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("en", tmp.path());
        assert_eq!(i18n.t("welcome"), "Welcome to Helios CLI");
    }

    #[test]
    fn test_t_french() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("fr", tmp.path());
        assert_eq!(i18n.t("welcome"), "Bienvenue dans Helios CLI");
    }

    #[test]
    fn test_fallback_to_english() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("fr", tmp.path());
        // "cli.version" is not in French, should fall back to English
        assert_eq!(i18n.t("cli.version"), "Version {version}");
    }

    #[test]
    fn test_missing_key_returns_key() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("en", tmp.path());
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_t_with_interpolation() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("en", tmp.path());
        let result = i18n.t_with("errors.file_not_found", &[("path", "/tmp/test.txt")]);
        assert_eq!(result, "File not found: /tmp/test.txt");
    }

    #[test]
    fn test_lang_method() {
        let tmp = TempDir::new().unwrap();
        setup_test_locales(tmp.path());
        let i18n = I18n::with_dir("fr", tmp.path());
        assert_eq!(i18n.lang(), "fr");
    }
}
