use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

use crate::monitors::{Monitor, add_monitor_to_group};
use crate::jobs::{Job, Status, return_all_jobs};
use crate::monitors::{find_monitor, return_all_monitors};

// monitor strcut for saving to toml
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub length: String,
    pub id: String,
    pub monitor_names: Vec<String>,
}
// for saving multiple monitors to toml
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GroupFile {
    pub groups: Vec<Group>
}
fn return_groups_toml() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("groups.toml")
}


// CREATES NEW GROUP + ADDS TO groups.toml
pub fn create_group(name: &str) {
    // check if new group name is taken
    if check_if_group_name_taken(name) {
        return;
    }
    // get new id = number of groups + 1
    let new_id = return_all_groups().len() + 1;
    // creates new group + add to groups.toml
    let new_group : Group = Group {
        name: name.to_string(),
        length: "0".to_string(),
        id: new_id.to_string(),
        monitor_names: Vec::new(),
    };
    save_group_credentials(&new_group);
}
// RETURNS TRUE IF GROUP NAME IS IN groups.toml, FALSE OTHERWISE
pub fn check_if_group_name_taken(name: &str) -> bool {
    let contents = match fs::read_to_string(return_groups_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => return false, // f the file doesn't exist, return false, end early
    };
    let data: GroupFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => return false, // if can't parse, return false, end early
    };
    for group in data.groups { // iterate over the monitors in the MonitorFile struct
        if group.name == name { // if the monitor name matches the name passed to the function
            return true; // return true, end early
        }
    }
    return false;
}
// RETURNS A Vec<Group> OF ALL GROUPS FOUND IN groups.toml
pub fn return_all_groups() -> Vec<Group> {
    let contents = match std::fs::read_to_string(return_groups_toml()) { // put the the fs read function thru a switch statement as an io check...
        Ok(content) => content, // if can read, set contets as contents, 
        Err(_) => panic!("Failed to read groups.toml"), // f the file doesn't exist, return false, end early
    };
    let data: GroupFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a MonitorFile struct
        Ok(data) => data, // if can parse, set data as the parsed data
        Err(_) => return Vec::new(), // if can't parse, return false, end early
    };
    return data.groups; // return all monitors
}
// SAVE Group TO groups.toml
fn save_group_credentials(new_group: &Group) {
    let mut data = match fs::read_to_string(return_groups_toml()) {
        Ok(content) => toml::from_str::<GroupFile>(&content).unwrap_or_default(), // If the file exists, parse it into a MonitorFile struct
        Err(_) => GroupFile::default(), // If the file doesn't exist, start with default [[monitors]] structure
    };

    data.groups.push(new_group.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write(return_groups_toml(), toml_string).unwrap();
}
// DELETE Group FROM groups.toml
pub fn delete_group(name: &str) {
    // removes group from groups.toml
    if check_if_group_name_taken(name) {
        let contents = match fs::read_to_string(return_groups_toml()) { // put the the fs read function thru a switch statement as an io check...
                Ok(content) => content, // if can read, set contets as contents, 
                Err(_) => return, // f the file doesn't exist, return false, end early
            };
            let mut data: GroupFile = match toml::from_str(&contents) { // parse the contents of monitors.toml into a mutable MonitorFile struct
                Ok(data) => data, // if can parse, set data as the parsed data
                Err(_) => return, // if can't parse, return false, end early
            };
            data.groups.retain(|g| g.name != name); // remove the monitor with the specified name from the monitors vector
            for i in 0..data.groups.len() { // resets all ids
                data.groups[i].id = i.to_string();
            }
            let toml_string = toml::to_string_pretty(&data).unwrap();
            fs::write(return_groups_toml(), toml_string).unwrap();
    }
}
pub fn return_monitors_in_group(cur_group_id: &usize) -> Vec<Monitor> { // takes the id of the current group id and returns a vector of monitors from that group
    // this stuff is for etting all monitors with the current group selection
    let monitors: Vec<Monitor> = return_all_monitors(); // gets all monitors from monitor.toml
    let jobs: Vec<Job> = return_all_jobs();
    let groups: Vec<crate::groups::Group> = return_all_groups(); // gets all groups in groups.toml
    let cur_group_name: &str = match cur_group_id { // sets group name
        0 => "Finished",
        1 => "Paused", 
        2 => "Running",
        3 => "", // unassigned groups have the name ""
        4 => "All", 
        _ => &groups[cur_group_id - 5].name, // custom group names start from id 5 upwards
    };
    let mut monitors_in_group = Vec::<Monitor>::new(); // htis new shit is needed for below push to work 
    match cur_group_name {
        "Finished" => {
            let jobs_to_show : Vec<&Job> = jobs.iter() // gets all jobs with finished status to find monitors
                .filter(|j| j.status == Status::Finished)
                .collect();
            for job in jobs_to_show { // find monitors with finihed status 
                monitors_in_group.push(find_monitor(job.name.clone()));
            }
        },
        "Paused" => {
            let jobs_to_show : Vec<&Job> = jobs.iter() // gets all jobs with finished status to find monitors
                .filter(|j| j.status == Status::Paused)
                .collect();
            for job in jobs_to_show { // find monitors with finihed status 
                monitors_in_group.push(find_monitor(job.name.clone()));
            }
        },
        "Running" => {
            let jobs_to_show : Vec<&Job> = jobs.iter() // gets all jobs with finished status to find monitors
                .filter(|j| j.status == Status::Running)
                .collect();
            for job in jobs_to_show { // find monitors with finihed status 
                monitors_in_group.push(find_monitor(job.name.clone()));
            }
        },
        "All" => {
            monitors_in_group = monitors.into_iter().collect();
        }, // idk if tihs collect shit is the best way to do it but it works !
        _ => {
            monitors_in_group = monitors
                .into_iter() // into inter instead of just iter to collect as Vec<Monitor> instead of <&Monitor> idk copied from substack
                .filter(|m| m.group_name == cur_group_name)
                .collect();
        },
    } 
    return monitors_in_group;
}
pub fn rename_group (old_group_id: &usize, new_group_name: &str) {
    if check_if_group_name_taken(new_group_name) {} else {
        // create the new group
        create_group(new_group_name);
        let group_id : usize = *old_group_id - 5;
        // asigns monitors to renamed group
        let monitors_to_rename : Vec<Monitor> = return_monitors_in_group(old_group_id); // fkkkkkkkkkkk
        monitors_to_rename.iter()
            .for_each(|m| add_monitor_to_group(&m.name, new_group_name));
        // delete old group
        let old_group_name = return_all_groups()[group_id].name.clone();
        delete_group(&old_group_name);

        // !!!! NOTE NEED A FUNCTION TO REORDER GROUPS SINCE THIS WILL JUST ADD THE RENAMED GROUP TO THE END OF THE LIST WHICH CAN BE ANNOYING SO FIX THIS IN FUTURE  
    }
    // gets all monitors with old group nmae
    // run monitors... add_monitor_to_group with new group name
    
}







