use serde::{Serialize, Deserialize};
use std::fs;
use crate::create::check_if_monitorname_taken;

use crate::monitors::{Monitor, MonitorFile};
use crate::jobs::{Job, JobFile};

pub fn delete_monitor(name: &str) {
    if check_if_monitorname_taken(name) {
        println!("-- Delete");
        // ASK YES OR NO STATEMENT IF USER WANTS TO DELETE
        println!("Are you certain that you want to delete this monitor? (Y)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        if input.trim() == "Y" {
            println!("deleting...");
            //delete from monitors.toml
            let contents = match fs::read_to_string("monitors.toml") { // put the the fs read function thru a switch statement as an io check...
                Ok(content) => content, // if can read, set contets as contents, 
                Err(_) => return, // f the file doesn't exist, return false, end early
            };
            let mut data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
                Ok(data) => data, // if can parse, set data as the parsed data
                Err(_) => return, // if can't parse, return false, end early
            };
            data.monitors.retain(|monitor| monitor.name != name); // remove the monitor with the specified name from the monitors vector
            let toml_string = toml::to_string_pretty(&data).unwrap();
            fs::write("monitors.toml", toml_string).unwrap();

            //delete from jobs.toml
            let contents = match fs::read_to_string("jobs.toml") {
                Ok(content) => content,
                Err(_) =>  return,
            };
            let mut data: JobFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
                Ok(data) => data, // if can parse, set data as the parsed data
                Err(_) => return, // if can't parse, return false, end early
            };
            data.jobs.retain(|job| job.name != name); // remove the job with the specified name from the jobs vector
            let toml_string = toml::to_string_pretty(&data).unwrap();
            fs::write("jobs.toml", toml_string).unwrap();
        } else {
            return;
        }
    } 
    else {
        println!("XXXX Monitor name does not exist: {}", name);
    }
}
