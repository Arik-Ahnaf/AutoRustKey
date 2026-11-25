//! Settings store. The original kept these under HKCU\Software\AutoHotkey\<section>;
//! here they live in ~/.config/autohotkey-ux/config.json as "section\key" entries.

use serde_json::{Map, Value};
use std::path::PathBuf;

pub struct Config {
    path: PathBuf,
    values: Map<String, Value>,
}

fn key(section: &str, name: &str) -> String {
    format!("{section}\\{name}")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let values = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        Self { path, values }
    }

    fn save(&self) {
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(&Value::Object(self.values.clone())) {
            let _ = std::fs::write(&self.path, text);
        }
    }

    pub fn get_str(&self, section: &str, name: &str, default: &str) -> String {
        match self.values.get(&key(section, name)) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Bool(b)) => (if *b { "1" } else { "0" }).to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => default.to_string(),
        }
    }

    pub fn get_bool(&self, section: &str, name: &str, default: bool) -> bool {
        match self.values.get(&key(section, name)) {
            Some(Value::Bool(b)) => *b,
            Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
            Some(Value::String(s)) => !(s.is_empty() || s == "0" || s.eq_ignore_ascii_case("false")),
            _ => default,
        }
    }

    pub fn set_str(&mut self, section: &str, name: &str, value: &str) {
        self.values.insert(key(section, name), Value::String(value.to_string()));
        self.save();
    }

    pub fn set_bool(&mut self, section: &str, name: &str, value: bool) {
        self.values.insert(key(section, name), Value::Bool(value));
        self.save();
    }

    pub fn delete(&mut self, section: &str, name: &str) {
        if self.values.remove(&key(section, name)).is_some() {
            self.save();
        }
    }
}

pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("AHK_UX_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("autohotkey-ux")
        .join("config.json")
}
