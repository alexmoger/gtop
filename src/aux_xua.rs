use std::process::{Command, Stdio};
use std::path::Path;
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


