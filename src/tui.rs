
use crate::aux::return_all_monitors;
use crate::aux::find_job;
use crate::monitors::Monitor;
use crate::get::update_monitor;

use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn debug_monitor(refresh_rate: u64) {
    println!("-- DEBUG TUI --");

    let interval = Duration::from_secs(refresh_rate);
    let mut next_time = Instant::now() + interval;
    
    loop {
        // update monitors
        let monitors_to_update : Vec<Monitor> = return_all_monitors();
        update_monitor(&monitors_to_update);

        print!("{esc}c", esc = 27 as char); // clears screen
        println!("{:?}", Instant::now());

        let monitordata = return_all_monitors();
        for monitor in monitordata {
            let jobdata = find_job(monitor.name.clone());
            println!("================================");
            /*let mut status : String = "".to_string();
            if jobdata.status == true {
                status = "[RUNNING]".to_string();
            } else if jobdata.status == false {
                if jobdata.cur_step == jobdata.max_step {
                    status = "[COMPLETED]".to_string();
                } else {
                    status = "[STOPPED]".to_string();
                }
            }*/

            println!("{}", monitor.name);
            println!("{}", jobdata.dir_size);
            let cur_time: f32 = (jobdata.cur_step.parse::<f32>().unwrap() * jobdata.time_step.parse::<f32>().unwrap()) / 1000000.0; // divide by 1mil to get microseconds
            let max_time: f32 = (jobdata.max_step.parse::<f32>().unwrap() * jobdata.time_step.parse::<f32>().unwrap()) / 1000000.0;
            let completion_percentage = jobdata.cur_step.parse::<f32>().unwrap() / jobdata.max_step.parse::<f32>().unwrap() * 100.0;
            println!("{:.3}/{:.1} us   ({:.2}%)", cur_time, max_time, completion_percentage);
        }
        println!("================================");

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
