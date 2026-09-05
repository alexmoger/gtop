// receives monitor name
// creates monitor struct with info from monitors.toml
// each command sshs into the pwd and gets deffnm.log or .mdp or whatever to get data

use crate::monitors::{Monitor, MonitorFile};
use crate::create::test_local_address;
use std::process::{Command, Stdio};
use crate::jobs::{Job, JobFile, Status, find_job, return_jobs_toml};
use std::fs;
use chrono;


//UPDATE FUNC
pub fn update_monitor(monitors: &Vec<Monitor>) {
    //println!("-- Update");
    for monitor in monitors {
        let mut cur_job = find_job(monitor.name.clone()); // gets the job struct for the monitor

        // update prev step before assigned new cur_step 
        cur_job.last_step = cur_job.cur_step.clone(); 

        let files = get_files(monitor); // vec of [mdp contents][log 10th last line]
        let max_steps = get_max_steps(files[0].clone()); // gets max steps from mdp file
        let timestep = get_timestep(files[0].clone()); // gets timestep from mdp file
        
        // CUR STEP STUFFS
        let mut cur_step = get_cur_step(files[1].clone()); // gets current step from log file, just the string from the log file line
        let search_xtc_pattern = format!("{}/{}.xtc", monitor.work_dir, monitor.deffnm); // /work_dir/deffnm.xtc check if the prod xtc doesnt exist 
        //      get job status first
        //          check if job status running or waiting
        let mut finished_cur_step = "_".to_string(); // get the entry for the log file as if the simulation was finished
        if test_local_address(&search_xtc_pattern) { //  check if xtc exists ;; idk if needs to be referenced since not using later ... fk  this rust language
            // xtc exists, then check if log file represents running log file or not (line = int or otherwise)
            match cur_step.parse::<i32>() { // attempt to parse log line as int .. 
                Ok(_) => cur_job.status = Status::Running, //  set status running if int
                Err(_) => { // if cant an int where the line for a running log file should be....
                    finished_cur_step = get_stopped_job_step(monitor); // need to run this here since no log file exists ina  waiting condition...
                    if finished_cur_step == max_steps { // if equal to max steps then jobs finished + IF NO LOG FILE EXISTS IE JOB WAITING !!!
                        cur_job.status = Status::Finished;
                    }  else {
                        cur_job.status = Status::Paused;
                    }
                },
            }       
            
        } else { cur_job.status = Status::Waiting; } // no xtc exists so waiting
        //           assign cur step given job status
        match cur_job.status {
            Status::Finished => cur_step = cur_job.max_step, // if finished, cur step = max step
            Status::Paused => cur_step = finished_cur_step, // if paused, get cur step from log file
            Status::Running => {}, // if running no nothing since cur step already obtained when variable initialised
            Status::Waiting => {}, // no cur step if job isnt running
        }

        
        cur_job.max_step = max_steps; // update the job struct with the new max steps
        cur_job.cur_step = cur_step; // update the job struct with the new current step
        cur_job.time_step = timestep; // update the job struct with the new timestep
        cur_job.last_updated = chrono::Local::now().to_rfc3339(); // update the job struct with the new last updated time
    
        update_job(&cur_job); // update the job struct in jobs.toml
    }
}

fn update_job(job: &Job) {
    // loads jobs.toml
    let mut data = match fs::read_to_string(return_jobs_toml()) {
        Ok(content) => toml::from_str::<JobFile>(content.as_str()).unwrap_or_default(), // If the file exists, parse it into a JobFile struct
        Err(_) => JobFile::default(), // If the file doesn't exist, start with default [[jobs]] structure
    };

    // Find the index of the job to update
    if let Some(index) = data.jobs.iter().position(|j| j.name == job.name) {
        data.jobs[index] = job.clone(); // Update the job at the found index
    } else {
        panic!("Job not found: {}", job.name); // If the job is not found, panic
    }

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write(return_jobs_toml(), toml_string).unwrap();
}


// GET FILES 
// takes a monitor and returns a vector of strings (contents of each file processed in this function)
pub fn get_files(monitor: &Monitor) -> Vec<String> {
    // gets whole mdp contents
    let mdp_path = format!("{}/{}.mdp", monitor.work_dir, monitor.deffnm); //  path/to/deffnm.mdp
    let mdp_contents = fs::read_to_string(&mdp_path).unwrap_or_else(|_| panic!("error: cant't find [{}: {}]", monitor.name, mdp_path)); // contents of deffnm.mdp << return 

    // returns equivalent of tail -n 10 | head -n 1 on deffnm.log
    let log_path = format!("{}/{}.log", monitor.work_dir, monitor.deffnm); //  path/to/deffnm.log
    let log_contents = fs::read_to_string(&log_path).unwrap_or_else(|_| String::new()); // contents of deffnm.log

    let mut log_lines : Vec<&str>;
    let mut log_step_line : String;
    if log_contents.is_empty() {
        log_lines = Vec::<&str>::new();
        log_step_line = "".to_string();
    } else {
        log_lines = log_contents.lines().collect(); // gets vec of log file lines
        log_step_line = log_lines[log_lines.len() - 10].to_string(); // gets line 10 from end (containing step in a running log file) << return 
    }

    // returns [mdp contents, log 10th last line]
    let contents : Vec<String> = vec![mdp_contents, log_step_line]; // returns vec of strings [mdp contents, log step line]
    return contents;
}


// GET MAX STEPS from mdp file 
fn get_max_steps(mdp_file: String) -> String{
    let nsteps_line = mdp_file
        .lines() // splits the string into an iterator of lines
        .find(|line| line.trim_start().starts_with("nsteps")) // finds the line that starts with "nsteps"
        .and_then(|line| line.split('=').nth(1)) // splits the line at the = and gets the second part 
        .map(|value| value.trim().split_whitespace().next()) // remove extra spaces, splits at whitespace and gets the first part
        .flatten() // flattens the Option<Option<&str>> to Option<&str>
        .map(|value| value.to_string()); // converts the &str to String
    return nsteps_line.unwrap_or_else(|| "0".to_string()); // returns the line or "0" if not found
}

// GET DT from mdp file
fn get_timestep(mdp_file: String) -> String{
    let dt_line = mdp_file
        .lines() // splits the string into an iterator of lines
        .find(|line| line.trim_start().starts_with("dt")) // finds the line that starts with "dt"
        .and_then(|line| line.split('=').nth(1)) // splits the line at the = and gets the second part
        .map(|value| value.trim().split_whitespace().next()) // remove extra spaces, splits at whitespace and gets the first part
        .flatten() // flattens the Option<Option<&str>> to Option<&str>
        .map(|value| value.to_string()); // converts the &str to String
    return dt_line.unwrap_or_else(|| "0".to_string()); // returns the line or "0" if not found ( for monitor stuff so it doesnt blow up)
}

// GET DIRECTORY SIZE from du output
fn get_dir_size(du_output: String) -> String {
    du_output
        .split_whitespace() // splits by any whitespace (space, tab, newline)
        .next() // gets the first element (the size)
        .unwrap_or("size not found")
        .to_string()
}

fn get_cur_step(log_file: String) -> String {
    log_file
        .split_whitespace() // splits by any whitespace (space, tab, newline)
        .next() // gets the first element (the current step)
        .unwrap_or("current step not found")
        .to_string()
}

// returns cur step when job has finished / stopped running (jobstatus == false, cur_step no longer on last line 4 or 5)
// TODO <<< make this the genreal way to get stuff, not the tail -n5 | head- n 1 method as above its too sloppy
fn get_stopped_job_step(monitor: &Monitor) -> String { // this is messy as hell
    let log_path = format!("{}/{}.log", monitor.work_dir, monitor.deffnm); //   path/to/deffnm.log
    let log_contents = fs::read_to_string(&log_path).unwrap_or_else(|_| panic!("no log file found")); // contents of deffnm.log
    let log_lines : Vec<&str> = log_contents.lines().collect(); // gets vec of log file lines
    let log_last_500_lines  = log_lines[(log_lines.len().saturating_sub(500))..].to_vec();

    // save index of each line with a Step 
    let mut step_indices : Vec<i32> = Vec::new(); // indices containing a step (index of lines contain("step") + 1)
    for i in 0..log_last_500_lines.len() { // iterate thru 0->length of lines
        if log_last_500_lines[i].contains("Step") { // if it contains "Step"...
            let step_index : i32 = <usize as TryInto<i32>>::try_into(i).unwrap() + 1; // no idea how this works the compiler said to use it \o/
            step_indices.push(step_index); // push index of line containig "Step" + 1
        }
    }
    // get last index + 1
    let last_index = step_indices[step_indices.len() - 1];
    let mut last_line : &str = "";
    for i in 0..log_last_500_lines.len() { // there has to be a better way to do this .........
        if <usize as TryInto<i32>>::try_into(i).unwrap() == last_index {
            last_line = log_last_500_lines[i];
        }
    }
    // split + get step + return
    last_line
        .split_whitespace() // splits by any whitespace (space, tab, newline)
        .next() // gets the first element (the current step)
        .unwrap_or("current step not found")
        .to_string()
}


