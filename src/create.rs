use std::{fs, env};
use std::path::Path;

use crate::groups::{return_all_groups,check_if_group_name_taken, create_group};
use crate::monitors::{Monitor, MonitorFile, return_all_monitors, return_monitors_toml, check_if_monitorname_taken};
use crate::jobs::{Job, JobFile, Status, return_jobs_toml};
use crate::get::update_monitor;
use crate::settings::{load_settings};

// Create monitor function
// takes four arguments, name for the monitor, the ssh address (usnm@host format), and working dir
pub fn create_monitor(name: &str, work_dir: &str, deffnm: &str, checkpoint_sub: &str) {
    // if default parsed for each argument, assign values based off settings file
    let settings= load_settings();
    let n_monitors = return_all_monitors().len();
    let name = match name { 
        "default" => { 
            let mut monitor_name_iterator = 0;
            let monitor_name;
            loop {
                let possible_name = format!("{}_{}", settings.default_monitorname, n_monitors + monitor_name_iterator);
                if check_if_monitorname_taken(&possible_name) {
                    monitor_name_iterator += 1;
                    continue;
                }
                monitor_name = possible_name;
                break;
            }         
            monitor_name   
        } // if name == default, use defaultmonitorname_nmonitors  
        _ => { if check_if_monitorname_taken(name) == false { name.to_string() } else { panic!("name taken"); } } // else return what the user wrote given that the name isnt already in monitors.toml 
     };
     let work_dir = match work_dir {
        "cur_dir" => env::current_dir().expect("cur_dir doesnt exist").display().to_string(),
        _ => work_dir.to_string(),
     };
     let deffnm = match deffnm {
        "default" => settings.default_deffnm,
        _ => deffnm.to_string()
     };
     let checkpoint_sub = match checkpoint_sub {
        "default" => settings.default_checkpointsub,
        _ => checkpoint_sub.to_string()
     };

    println!("attempting to create a new monitor");
    println!("---- Monitor name: {}", name);
    println!("---- Work Directory: {}", work_dir);
    println!("---- Prefix: {}", deffnm);
    println!("---- Checkpoint Submission Script: {}", checkpoint_sub);


     // add command needs an mdp in the directory (pwd/deffnm.mdp) to succesfully create a monitor 
    let search_log_pattern: String = format!("{}/{}.log", work_dir, deffnm);
    println!("checking for {search_log_pattern}");
    if test_local_address(&search_log_pattern) { // checks if work_dir exists
        println!("validation passed");
        let new_monitor = Monitor { // creates the monitor to add using the Monitor struct
            name: name.to_string(),
            work_dir: work_dir.to_string(),
            deffnm: deffnm.to_string(),
            checkpoint_sub: checkpoint_sub.to_string(),
            group_name: "".to_string(),
        };
        save_monitor_credentials(&new_monitor); // save monitor credentials to /gtop/monitors/name.toml
        save_job_credentials(&name);
        update_monitor(&vec![new_monitor]); // first update auto done to initialise monitor
    } else { println!("no mdp found in directory...")}    
}

pub fn create_monitors_from_parent(children: Vec<String>, group: &str, name: &str, work_dir: &str, deffnm: &str, checkpoint_sub: &str) {
    // check if group exists, if not create a new group using group_i where i  = Ngroups, if it does then set group to group
    let n_groups = return_all_groups().len();
    let group = match group {
    "default" => { // if left unassigned then create a new group
        let mut group_name_iterator = 0;
        let group_name;
        loop {
            let possible_name = format!("group_{}", n_groups + group_name_iterator);
            if check_if_group_name_taken(&possible_name) {
                group_name_iterator += 1;
                continue;
            }
            group_name = possible_name;
            break;
        }         
        group_name   
    },
    _ => if check_if_group_name_taken(group) == false { group.to_string() } else { panic!("group name already taken");}
    };
    // create group
    create_group(&group);

    // if default parsed for each argument, assign values based off settings file
    let settings= load_settings();
    let name = match name { 
        "default" => format!("{}_monitor", group).to_string(), // if name == default, use defaultmonitorname_nmonitors  
        _ => { if check_if_monitorname_taken(name) == false { name.to_string() } else { panic!("name taken"); } } // else return what the user wrote given that the name isnt already in monitors.toml 
     };
     let work_dir = match work_dir {
        "cur_dir" => env::current_dir().expect("cur_dir doesnt exist").display().to_string(),
        _ => work_dir.to_string(),
     };
     let deffnm = match deffnm {
        "default" => settings.default_deffnm,
        _ => deffnm.to_string()
     };
     let checkpoint_sub = match checkpoint_sub {
        "default" => settings.default_checkpointsub,
        _ => checkpoint_sub.to_string()
     };

     // get number
     let n_monitors_to_create = children.len();
     println!("making {} children monitors: {:?}", n_monitors_to_create, children);

     let mut iterator = 1;
     for child in children {
        let child_name = format!("{}_{}", name, iterator);
        let child_workdir = format!("{}/{}", work_dir, child);

        println!("attempting to create a new monitor");
        println!("---- Monitor name: {}", child_name);
        println!("---- Work Directory: {}", child_workdir);
        println!("---- Prefix: {}", deffnm);
        println!("---- Checkpoint Submission Script: {}", checkpoint_sub);

        // add command needs an mdp in the directory (pwd/deffnm.mdp) to succesfully create a monitor 
        let search_log_pattern: String = format!("{}/{}/{}.log", work_dir, child, deffnm);
        println!("checking for {search_log_pattern}");
        if test_local_address(&search_log_pattern) { // checks if work_dir exists
            println!("validation passed");
            let new_monitor = Monitor { // creates the monitor to add using the Monitor struct
                name: child_name.to_string(),
                work_dir: child_workdir.to_string(),
                deffnm: deffnm.to_string(),
                checkpoint_sub: checkpoint_sub.to_string(),
                group_name: group.to_string(),
            };
            save_monitor_credentials(&new_monitor); // save monitor credentials to /gtop/monitors/name.toml
            save_job_credentials(&child_name);
            update_monitor(&vec![new_monitor]); // first update auto done to initialise monitor


        } else { println!("no mdp found in directory...")} 
        iterator += 1;
     }
}

// Saves the monitor credentials to monitors.toml
fn save_monitor_credentials(new_monitor: &Monitor) {
    println!("saving monitor credentials to monitors.toml");

    let mut data = match fs::read_to_string(return_monitors_toml()) {
        Ok(content) => toml::from_str::<MonitorFile>(&content).unwrap_or_default(), // If the file exists, parse it into a MonitorFile struct
        Err(_) => MonitorFile::default(), // If the file doesn't exist, start with default [[monitors]] structure
    };

    data.monitors.push(new_monitor.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write(return_monitors_toml(), toml_string).unwrap();
}


pub fn save_job_credentials(name: &str,) {
    println!("saving job credentials to jobs.toml");

    let new_job = Job {
        name: name.to_string(),
        max_step: "0".to_string(),
        cur_step: "0".to_string(),
        time_step: "0".to_string(),
        last_updated: "0".to_string(),
        last_step: "0".to_string(),
        status: Status::Waiting,
        nmpi: "0".to_string(),
        nomp: "0".to_string(),
        gpu_enbaled: "0".to_string(),
    };

    let mut data = match fs::read_to_string(return_jobs_toml()) {
        Ok(content) => toml::from_str::<JobFile>(&content).unwrap_or_default(), // If the file exists, parse it into a JobFile struct
        Err(_) => JobFile::default(), // If the file doesn't exist, start with default [[jobs]] structure
    };

    data.jobs.push(new_job.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write(return_jobs_toml(), toml_string).unwrap();
}

pub fn test_local_address(search: &str) -> bool {
    /*match Command::new("sh")
        .arg("-c")
        .arg(format!("ls {} >/dev/null 2>&1", search)) // performs an ls search for files specified by workdir/deffnm*, dev null stuff silences output
        .status() // returns Result<ExitStatus, io::Error>, interpreted by the Ok() function
    {
        Ok(status) => status.success(), // if ssh command returns exit code 0, return true
        Err(_) => false, // if ssh command fails to run, return false
    }*/
    Path::new(search).exists()
}
