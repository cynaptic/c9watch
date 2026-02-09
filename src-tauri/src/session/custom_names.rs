use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CustomNames {
    pub names: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CustomTitles {
    pub titles: HashMap<String, String>,
}

impl CustomNames {
    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    fn get_path() -> PathBuf {
        let home = dirs::home_dir().expect("Failed to get home directory");
        home.join(".claude").join("session-monitor-names.json")
    }

    pub fn get(&self, session_id: &str) -> Option<&String> {
        self.names.get(session_id)
    }

    pub fn set(&mut self, session_id: String, name: String) {
        self.names.insert(session_id, name);
    }
}

impl CustomTitles {
    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    fn get_path() -> PathBuf {
        let home = dirs::home_dir().expect("Failed to get home directory");
        home.join(".claude").join("session-monitor-titles.json")
    }

    pub fn get(&self, session_id: &str) -> Option<&String> {
        self.titles.get(session_id)
    }

    pub fn set(&mut self, session_id: String, title: String) {
        self.titles.insert(session_id, title);
    }
}
