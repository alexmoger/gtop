use serde::{Serialize, Deserialize};

// monitor strcut for saving to toml
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Monitor {
    pub name: String,
    pub ssh_address: String,
    pub work_dir: String,
    pub deffnm: String,
    pub checkpoint_sub: String
    // slurm job id
}
// for saving multiple monitors to toml
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MonitorFile {
    pub monitors: Vec<Monitor>
}

// todo -- add impl Job new() to create new job+moniotr
