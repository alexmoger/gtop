use crate::aux_xua::test_ssh_connection;
use crate::aux_xua::{test_local_address, check_if_dir_exists };
use serde::{Serialize, Deserialize};
use std::fs;

use crate::monitors::{Monitor, MonitorFile};
use crate::jobs::{Job, JobFile, Status};
use crate::get::update_monitor;

// Create monitor function
// takes four arguments, name for the monitor, the ssh address (usnm@host format), and working dir
pub fn create_monitor(name: &str, ssh_address: &str, work_dir: &str, deffnm: &str, checkpoint_sub: &str) {
    println!("-- Create");
    println!("---- Monitor name: {}", name);
    println!("---- SSH Address: {}", ssh_address);
    println!("---- Work Directory: {}", work_dir);
    println!("---- Prefix: {}", deffnm);
    println!("---- Checkpoint Submission Script: {}", checkpoint_sub);

    // see if monitor name already exists
    if check_if_monitorname_taken(name) {
        eprintln!("XXXX Monitor name already exists: {}", name);
        return;
    }

    let search_mdp_pattern: String = format!("{}/{}.mdp", work_dir, deffnm); // string containing workdir  AND CONTAINS DEFFNM.MDP !!!!
    let search_tpr_pattern = format!("{}/{}.tpr", work_dir, deffnm);     // string containing workdir/deffnm*
    if ssh_address == "@" { // if local job, no ssh just need to validate that sim files are present
        println!("checking for {search_mdp_pattern}");
        if test_local_address(&search_mdp_pattern) { // checks if work_dir exists
            println!("validation passed");
            let new_monitor = Monitor { // creates the monitor to add using the Monitor struct
                name: name.to_string(),
                ssh_address: ssh_address.to_string(),
                work_dir: work_dir.to_string(),
                deffnm: deffnm.to_string(),
                checkpoint_sub: checkpoint_sub.to_string(),
                group_name: "".to_string(),
            };
            save_monitor_credentials(&new_monitor); // save monitor credentials to /gtop/monitors/name.toml
            save_job_credentials(name);
            update_monitor(&vec![new_monitor]); // first update auto done to initialise monitor
        } else { println!("no mdp found in directory...")}
    } else {
        if test_ssh_connection(ssh_address, &search_mdp_pattern) { // test if can connect interactively to ssh_address (uname@host format) and find items listed in workdir/deffnm*
            println!("validation test passed for monitor: {}", name);
            let new_monitor = Monitor { // creates the monitor to add using the Monitor struct
                name: name.to_string(),
                ssh_address: ssh_address.to_string(),
                work_dir: work_dir.to_string(),
                deffnm: deffnm.to_string(),
                checkpoint_sub: checkpoint_sub.to_string(),
                group_name: "".to_string()
            };
            save_monitor_credentials(&new_monitor); // save monitor credentials to /gtop/monitors/name.toml
            save_job_credentials(name);
            update_monitor(&vec![new_monitor]); // first update auto done to initialise monitor
        } else {
            eprintln!("validation test failed for monitor: {}", name);
        }
    }
    
}

// Checks if the monitor name is already taken (in monitors.toml), returns true if taken, false if not
pub fn check_if_monitorname_taken(name: &str) -> bool {
    let contents = match fs::read_to_string("monitors.toml") { // put the the fs read function thru a switch statement as an io check...
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

// Saves the monitor credentials to monitors.toml
fn save_monitor_credentials(new_monitor: &Monitor) {
    println!("saving monitor credentials to monitors.toml");

    let mut data = match fs::read_to_string("monitors.toml") {
        Ok(content) => toml::from_str::<MonitorFile>(&content).unwrap_or_default(), // If the file exists, parse it into a MonitorFile struct
        Err(_) => MonitorFile::default(), // If the file doesn't exist, start with default [[monitors]] structure
    };

    data.monitors.push(new_monitor.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write("monitors.toml", toml_string).unwrap();
}


pub fn save_job_credentials(name: &str,) {
    println!("saving job credentials to jobs.toml");

    let new_job = Job {
        name: name.to_string(),
        max_step: "0".to_string(),
        cur_step: "0".to_string(),
        time_step: "0".to_string(),
        dir_size: "0".to_string(),
        last_updated: "0".to_string(),
        last_step: "0".to_string(),
        status: Status::Waiting,
    };

    let mut data = match fs::read_to_string("jobs.toml") {
        Ok(content) => toml::from_str::<JobFile>(&content).unwrap_or_default(), // If the file exists, parse it into a JobFile struct
        Err(_) => JobFile::default(), // If the file doesn't exist, start with default [[jobs]] structure
    };

    data.jobs.push(new_job.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write("jobs.toml", toml_string).unwrap();
}