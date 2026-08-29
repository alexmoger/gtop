use std::process::{Command, Stdio};
use crate::monitors::{Monitor, MonitorFile};
use crate::jobs::{Job, JobFile};
use crate::create::save_job_credentials;

// Tests SSH connection to address + whether files exist in workdir, return bool (True = connect succesfully)
// takes 2 arguments, the ssh address (usnm@host format) and the cd command to search for files 
pub fn test_ssh_connection(address: &str, search: &str) -> bool {
    println!("attempting to connect...");
    match Command::new("ssh") // match = rust switch statement
        .arg(address)
        .stdin(Stdio::inherit()) // for running an interactive ssh
        .stdout(Stdio::inherit()) 
        .stderr(Stdio::inherit())
        .arg(format!("ls {} >/dev/null 2>&1", search)) // performs an ls search for files specified by workdir/deffnm*, dev null stuff silences output
        .status() // returns Result<ExitStatus, io::Error>, interpreted by the Ok() function
    {
        Ok(status) => status.success(), // if ssh command returns exit code 0, return true
        Err(_) => false, // if ssh command fails to run, return false
    }
}

// tests if sim files (deffnm.*) are present in a working dir given no ssh address 
// for checking if a file is in a dir
pub fn test_local_address(search: &str) -> bool {
    match Command::new("sh")
        .arg("-c")
        .arg(format!("ls {} >/dev/null 2>&1", search)) // performs an ls search for files specified by workdir/deffnm*, dev null stuff silences output
        .status() // returns Result<ExitStatus, io::Error>, interpreted by the Ok() function
    {
        Ok(status) => status.success(), // if ssh command returns exit code 0, return true
        Err(_) => false, // if ssh command fails to run, return false
    }
}

// checks if a directory exists
// for checking if a directory exists
pub fn check_if_dir_exists(search: &str) -> bool {
    match Command::new("sh")
        .arg("-c")
        .arg(format!("[ -d {} ] && echo true ", search)) // tests if dir exists with [ -d /dir/ ], prints true if true
        .output() // returns output of abovee
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "true", // if ssh command returns exit code 0, return true
        Err(_) => false, // if ssh command fails to run, return false
    }
}

pub fn find_job(name: String) -> Job {
    let contents = match std::fs::read_to_string("jobs.toml") { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read jobs.toml"), // f the file doesn't exist, return false, end early
    };
    let data: JobFile = match toml::from_str(&contents) { // parse the contents of jobs.toml into a JobFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(data) =>  panic!("wtf"), // if job isnt present for whatever reason, create a new empty job struct and save it to jobs.toml
    };
    for job in data.jobs { // iterate over the jobs in the JobFile struct
        if job.name == name { // if the job name matches the name passed to the function
            return job; // return true, end early
        }
    }
    panic!("Job not found: {}", name);
}

pub fn return_all_jobs() -> Vec<Job> {
    let contents = match std::fs::read_to_string("jobs.toml") { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read jobs.toml"), // f the file doesn't exist, return false, end early
    };
    let data: JobFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("Failed to parse monitors.toml"), // if can't parse, return false, end early
    };
    return data.jobs; // return all monitors
}
