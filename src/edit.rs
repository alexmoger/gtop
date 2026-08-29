use crate::monitors::find_monitor;
use crate::delete::delete_monitor;
use crate::create::create_monitor;
use crate::create::check_if_monitorname_taken;

pub fn edit_monitor(name: String, feature: String, new_entry: String) {  
    let things_to_edit = ["name", "work_dir", "deffnm", "checkpoint_sub"]; // list of possible features
    let mut monitor = find_monitor(name); // gets monitor
    if (feature.as_str() == "name") {
        if (check_if_monitorname_taken(&new_entry)) {
            panic!("monitor name already taken");
        }
    }
    match feature.as_str() { // there has to be abetter way of doing this x)
        "name" => { monitor.name = new_entry; }
        "work_dir" => { monitor.work_dir = new_entry; }
        "deffnm" => {monitor.deffnm = new_entry; }
        "checkpoint_sub" => {monitor.checkpoint_sub = new_entry; }
        _ => panic!("invalud feature name, pls enter {:?}", things_to_edit)
    }
    // check if new monitor has correct values (deffnm exists? pwd exists)
    // deletes old monitor
    // add edited monitor
    //create_monitor(&monitor.name, &monitor.ssh_address, &monitor.work_dir, &monitor.deffnm, &monitor.checkpoint_sub, &group_name);
}