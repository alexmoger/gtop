use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Status {
    #[default]
    Waiting,
    Running,
    Paused,
    Finished,
}

// job strcut for saving to toml
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub name: String,
    pub max_step: String,
    pub cur_step: String,
    pub time_step: String,
    pub last_updated: String,
    pub last_step: String,
    pub status: Status, // true = running  
    pub nmpi: String,
    pub nomp: String,
    pub gpu_enbaled : String,
}
// for saving multiple jobs to toml
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct JobFile {
    #[serde(default)] // returns a default vec when jobs.toml is empty
    pub jobs: Vec<Job>
}
pub fn return_jobs_toml() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("jobs.toml")
}

// todo -- add impl new () to create new job + jobfile

// RETURNS A Job GIVEN A MONITOR NAME
pub fn find_job(name: String) -> Job {
    let contents = match std::fs::read_to_string( return_jobs_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read jobs.toml"), // f the file doesn't exist, return false, end early
    };
    let data: JobFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("Failed to parse jobs.toml"), // if can't parse, return false, end early
    };
    for job in data.jobs { // iterate over the monitors in the MonitorFile struct
        if job.name == name { // if the monitor name matches the name passed to the function
            return job; // return true, end early
        }
    }
    panic!("Job not found: {}", name);
}
pub fn return_all_jobs() -> Vec<Job> {
    let contents = match std::fs::read_to_string(return_jobs_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read jobs.toml"), // f the file doesn't exist, return false, end early
    };
    let data: JobFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => panic!("Failed to parse jobs.toml"), // if can't parse, return false, end early
    };
    return data.jobs; // return all monitors
}
