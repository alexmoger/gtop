use serde::{Serialize, Deserialize};
use std::fs;

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
    let contents = match fs::read_to_string("groups.toml") { // put the the fs read function thru a switch statement as an io check...
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
    let contents = match std::fs::read_to_string("groups.toml") { // put the the fs read function thru a switch statement as an io check...
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
    let mut data = match fs::read_to_string("groups.toml") {
        Ok(content) => toml::from_str::<GroupFile>(&content).unwrap_or_default(), // If the file exists, parse it into a MonitorFile struct
        Err(_) => GroupFile::default(), // If the file doesn't exist, start with default [[monitors]] structure
    };

    data.groups.push(new_group.clone()); // could probably jsut clone before instead of reference + clone but wtf

    let toml_string = toml::to_string_pretty(&data).unwrap();
    fs::write("groups.toml", toml_string).unwrap();
}
// DELETE Group FROM groups.toml
pub fn delete_group(name: &str) {
    // removes group from groups.toml
    if check_if_group_name_taken(name) {
        let contents = match fs::read_to_string("groups.toml") { // put the the fs read function thru a switch statement as an io check...
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
            fs::write("groups.toml", toml_string).unwrap();
    }
    // go through all monitors in monitor_names and set to unnassigned (no group)
    // get group
    // for each monito rin group.monitornames
}









