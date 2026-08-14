use serde::{Serialize, Deserialize};

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
    pub dir_size: String,
    pub last_updated: String,
    pub last_step: String,
    pub status: Status, // true = running  
}
// for saving multiple jobs to toml
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct JobFile {
    pub jobs: Vec<Job>
}

// todo -- add impl new () to create new job + jobfile
