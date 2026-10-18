use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

use crate::groups::check_if_group_name_taken;
use crate::jobs::{JobFile, return_jobs_toml};

// monitor strcut for saving to toml
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Monitor {
    pub name: String,
    pub work_dir: String,
    pub deffnm: String,
    pub checkpoint_sub: String,
    pub group_name: String,
}
// for saving multiple monitors to toml
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MonitorFile {
    #[serde(default)] // returns a default vec when monitors.toml is empty
    pub monitors: Vec<Monitor>
}
pub fn return_monitors_toml() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("monitors.toml")
}


// RETURNS A Vec<Monitor> OF ALL MONITORS IN monitors.toml
pub fn return_all_monitors()  -> Vec<Monitor> {
    let contents = match fs::read_to_string(return_monitors_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read monitors.toml"), // f the file doesn't exist, return false, end early
    };
    let data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("Failed to parse monitors.toml"), // if can't parse, return false, end early
    };
    return data.monitors; // return all monitors
}

// TAKES A MONITOR INPUT AND REPLACES THE SAME monitor.name IN monitors.toml 
pub fn update_monitor(monitor: Monitor) {
    // loads monitors.toml
    let mut data = match fs::read_to_string(return_monitors_toml()) {
        Ok(content) => toml::from_str::<MonitorFile>(content.as_str()).unwrap_or_default(), // If the file exists, parse it into a JobFile struct
        Err(_) => MonitorFile::default(), // If the file doesn't exist, start with default [[jobs]] structure
    };
    // Find the index of the job to update
    if let Some(index) = data.monitors.iter().position(|m| m.name == monitor.name) { // go trhu monitors and find monitor.name
        data.monitors[index] = monitor.clone(); // Update the job at the found index
    } else {
        panic!("Monitor not found: {}", monitor.name); // If the job is not found, panic
    }
    // save 
    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write("monitors.toml", toml_string).unwrap(); 
}

// RETURNS A Monitor GIVEN A MONITOR NAME
pub fn find_monitor(name: String) -> Monitor {
    let contents = match std::fs::read_to_string(return_monitors_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read monitors.toml"), // f the file doesn't exist, return false, end early
    };
    let data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("Failed to parse monitors.toml"), // if can't parse, return false, end early
    };
    for monitor in data.monitors { // iterate over the monitors in the MonitorFile struct
        if monitor.name == name { // if the monitor name matches the name passed to the function
            return monitor; // return true, end early
        }
    }
    panic!("Monitor not found: {}", name);
}

// SETS THE group_name OF TARGET MONITOR
pub fn add_monitor_to_group(monitorname: &str, groupname: &str) {
    let mut monitor = find_monitor(monitorname.to_string());
    if check_if_group_name_taken(groupname) || groupname == "" { // if groupname exists in groups.toml or is blank ie deallocate
        monitor.group_name = groupname.to_string(); // update new monitor with new groupname
    }
    // update monitors.toml with new monitor
    update_monitor(monitor);
    //UPDATE GROUP by added monitor name to group
}

pub fn delete_monitor(name: &str) {
    if check_if_monitorname_taken(name) {
        //println!("-- Delete");
        // ASK YES OR NO STATEMENT IF USER WANTS TO DELETE
        //println!("Are you certain that you want to delete this monitor? (Y)");
        //let mut input = String::new();
        let input = "Y";
        if input.trim() == "Y" {
            //delete from monitors.toml
            let contents = match fs::read_to_string(return_monitors_toml()) { // put the the fs read function thru a switch statement as an io check...
                Ok(content) => content, // if can read, set contets as contents, 
                Err(_) => return, // f the file doesn't exist, return false, end early
            };
            let mut data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
                Ok(data) => data, // if can parse, set data as the parsed data
                Err(_) => return, // if can't parse, return false, end early
            };
            data.monitors.retain(|monitor| monitor.name != name); // remove the monitor with the specified name from the monitors vector
            let toml_string = toml::to_string_pretty(&data).unwrap();
            fs::write(return_monitors_toml(), toml_string).unwrap();

            //delete from jobs.toml
            let contents = match fs::read_to_string(return_jobs_toml()) {
                Ok(content) => content,
                Err(_) =>  return,
            };
            let mut data: JobFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
                Ok(data) => data, // if can parse, set data as the parsed data
                Err(_) => return, // if can't parse, return false, end early
            };
            data.jobs.retain(|job| job.name != name); // remove the job with the specified name from the jobs vector
            let toml_string = toml::to_string_pretty(&data).unwrap();
            fs::write(return_jobs_toml(), toml_string).unwrap();
        } else {
            return;
        }
    } 
    else {
        println!("XXXX Monitor name does not exist: {}", name);
    }
}


// Checks if the monitor name is already taken (in monitors.toml), returns true if taken, false if not
pub fn check_if_monitorname_taken(name: &str) -> bool {
    let contents = match fs::read_to_string(return_monitors_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => return false, // f the file doesn't exist, return false, end early
    };
    let data: MonitorFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => return false, // if can't parse, return false, end early
    };
    for monitor in data.monitors { // iterate over the monitors in the MonitorFile struct
        if monitor.name == name { // if the monitor name matches the name passed to the function
            return true; // return true, end early
        }
    }
    return false;
}