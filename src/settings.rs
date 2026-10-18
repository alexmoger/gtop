use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub default_deffnm: String,
    pub default_checkpointsub: String,
    pub default_monitorname: String,
}
#[derive(Serialize, Deserialize)]
pub struct SettingsFile {
    settings: Vec<Settings>,
}

fn return_settings_toml() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("settings.toml")
}
pub fn load_settings() -> Settings { // returns the settings from settings.toml
    let contents = match std::fs::read_to_string(return_settings_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read settings.toml"), // f the file doesn't exist, return false, end early
    };
    let mut data: SettingsFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("wtf"), // if can't parse, return false, end early
    };
    return data.settings.remove(0); // return all monitors
}