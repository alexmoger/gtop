use serde::{Serialize, Deserialize};
use std::fs;

use crate::monitors::{Monitor, MonitorFile};

pub fn list_monitors() {
    println!("-- List");
    let contents = match fs::read_to_string("monitors.toml") { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => {return}
    };
    let data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => {return} // if can't parse, return false, end early
    };
    for monitor in data.monitors {
        println!("Name: {}", monitor.name);
        // PRINT STATUS
        println!("SSH Address: {}", monitor.ssh_address);
        println!("Work Directory: {}", monitor.work_dir);
        println!("Definition File: {}", monitor.deffnm);
        println!("Checkpoint Script: {}", monitor.checkpoint_sub);
        println!();
    }
}
