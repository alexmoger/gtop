
use crate::monitors::{Monitor, MonitorFile};
use crate::create::test_local_address;
use std::process::{Command, Stdio};
use crate::jobs::{Job, JobFile};
use crate::monitors::find_monitor;
use std::fs;
use chrono;

pub fn restart_monitor(monitor_name: String) {
    println!("-- Restart");
    let monitor_to_restart: Monitor = find_monitor(monitor_name);
    let checkpoint_script = format!("{}{}", monitor_to_restart.work_dir, monitor_to_restart.checkpoint_sub);
    // tests if checkpoint script exists
    if test_local_address(&checkpoint_script) {
        println!("running: sbatch {} WIP", checkpoint_script);
    } else {
        println!("no checkpoint script found");
    }
}