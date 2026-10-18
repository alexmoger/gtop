// receives monitor name
// creates monitor struct with info from monitors.toml
// each command sshs into the pwd and gets deffnm.log or .mdp or whatever to get data

use crate::monitors::{Monitor};
use crate::jobs::{Job, JobFile, Status, find_job, return_jobs_toml};
use std::fs;

//UPDATE FUNC
pub fn update_monitor(monitors: &Vec<Monitor>) {
    //println!("-- Update");
    for monitor in monitors {
        let mut cur_job = find_job(monitor.name.clone()); // gets the job struct for the monitor

        // update prev step before assigned new cur_step 
        cur_job.last_step = cur_job.cur_step.clone(); 

        // get values from log file
        let log_contents = read_from_logfile(monitor, true); // SHOULD ADD A THING TO MAKE THIS TRUE FALSE BUT FKKKK 2 LAZY SO DOES  AFULL READ EACH UPDATE CYCLE V WIP NEED 2 CLEAN UP X_x 
        // variablise output of read_from_logfile
        let gpu_enbaled = &log_contents[0];
        let dt = &log_contents[1];
        let max_steps = &log_contents[2];
        let n_mpi = &log_contents[3];
        let n_omp = &log_contents[4];
        let cur_step = &log_contents[5];
        let logfile_finished  = &log_contents[6];

        // set values in return obj
        cur_job.cur_step = cur_step.clone();
        cur_job.time_step = dt.clone();
        cur_job.max_step = max_steps.clone();
        cur_job.gpu_enbaled = gpu_enbaled.clone();
        cur_job.nmpi = n_mpi.clone();
        cur_job.nomp = n_omp.clone();

        // get status
        cur_job.status =
            if cur_job.cur_step == cur_job.max_step { Status::Finished } 
            else if logfile_finished == "1" { Status::Paused }  
            else { Status::Running }; // cant end on an else if statement x(

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

// returns a STRING vec with 6 entries (extracted from log file + processed)
// [
//      0 GPU enabled (enabled/disabled)
//      1 dt
//      2 nsteps
//      3 n MPI processes
//      4 n OMP threads per MPI process
//      5 cur steps 
//      6 logfile finished
// ]
pub fn read_from_logfile(monitor: &Monitor, full_check: bool) -> Vec<String> {
    let mut return_vec : Vec<String> = vec!["0".to_string(); 7]; // entries as described above

    let log_path = format!("{}/{}.log", monitor.work_dir, monitor.deffnm);
    let log_contents = fs::read_to_string(&log_path).unwrap_or_default();

    if full_check // run full read on monitor creation 
    {
        // check if GPU enabled or disabled
        let gpu_line : Option<&str> = log_contents
            .lines()
            .find(|l| l.starts_with("GPU support:"))
            .and_then(|l| l.strip_prefix("GPU support:"))
            .map(str::trim);
        return_vec[0] = gpu_line.unwrap_or("disabled").to_string(); 

        // find dt, nsteps
        let dt_line : Option<&str> = log_contents
            .lines()
            .find(|l| l.trim_start().starts_with("dt ")) // one space after dt
            .and_then(|l| l.split_once("="))
            .map(|(_,val)| val.trim());
        let nsteps_line : Option<&str> = log_contents
            .lines()
            .find(|l| l.trim_start().starts_with("nsteps"))
            .and_then(|l| l.split_once("="))
            .map(|(_,val)| val.trim());
        return_vec[1] = dt_line.unwrap_or("0").to_string();
        return_vec[2] =  nsteps_line.unwrap_or("0").to_string();

        // find n MPI processed, n OMP threads per MPI process
        let mpi: Option<&str> = log_contents
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("Using ")?
                    .strip_suffix(" MPI processes")
                    .map(str::trim)
            });
        return_vec[3] = mpi.unwrap_or("0").to_string();
        let omp : Option<&str> = log_contents
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("Using ")?
                    .strip_suffix("OpenMP thread per MPI process")
                    .map(str::trim)
            });
        return_vec[4] = omp.unwrap_or("0").to_string();

        //get cur steps 
        let mut line_after_header = false; // set true if reading line after "Step <whitesapce> Time"
        let mut last_data : Option<&str> = None;
        for l in log_contents.lines() { // gets last <stepvalue> <whitespace> <timevalue> line
            if line_after_header {
                last_data = Some(l); // overwrites
                line_after_header = false;
            }
            let mut contents = l.split_whitespace();
            if contents.next() == Some("Step") && contents.next() == Some("Time") && contents.next().is_none() {
                line_after_header = true; // reads step value from next iterated line
            }
        }
        if let Some(line) = last_data { // if last_data != None, put values into line
            let mut cols = line.split_whitespace();
            let step: Option<&str> = cols.next();
            return_vec[5] = step.unwrap_or("0").to_string();
        }
        
        // get if log file has terminated or not (running log file wont have the postamble)
        let is_finished: bool = log_contents // returns true if finished 
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .is_some_and(|l| l.trim_start().starts_with("Finished mdrun on rank"));
        if is_finished { return_vec[6] = "1".to_string(); } else { return_vec[6] = "0".to_string(); }
    }
    // CURRENTLY NOT CALLED SINCE UPDATES ALL VALUES AT EVERY CHECK SHOULD CHANGE THIS !
     else // update check -- CURRENTLY NOT USED X_X    
    { 
        //get cur steps 
        let mut line_after_header = false; // set true if reading line after "Step <whitesapce> Time"
        let mut last_data : Option<&str> = None;
        for l in log_contents.lines() { // gets last <stepvalue> <whitespace> <timevalue> line
            if line_after_header {
                last_data = Some(l); // overwrites
                line_after_header = false;
            }
            let mut contents = l.split_whitespace();
            if contents.next() == Some("Step") && contents.next() == Some("Time") && contents.next().is_none() {
                line_after_header = true; // reads step value from next iterated line
            }
        }
        if let Some(line) = last_data { // if last_data != None, put values into line
            let mut cols = line.split_whitespace();
            let step: Option<&str> = cols.next();
            return_vec[5] = step.unwrap_or("0").to_string();
        }
    }

    return return_vec;
}

