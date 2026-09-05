use clap::{Parser, Subcommand};
use std::io;
use std::time::{Duration, Instant};

mod aux_xua;
mod monitors;
mod create;
mod delete;
mod list;
mod get;
mod jobs;
mod restart;
mod edit;
mod app;
mod groups;
mod settings;

use create::{create_monitor, create_monitors_from_parent};
use delete::delete_monitor;
use list::list_monitors;
use get::update_monitor;
use restart::restart_monitor;
use edit::edit_monitor;
use groups::{create_group, delete_group};
use monitors::{find_monitor, add_monitor_to_group, return_all_monitors};
use settings::{Settings, load_settings};

use crate::monitors::Monitor;
use crate::app::App;
use crate::app::CurrentScreen;

// tui stuff
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};


#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    // create monitor
    Add {
        #[arg(short='n', long, default_value = "default")] // if left empty 
        name: String,
        #[arg(long, default_value = "cur_dir")]
        work_dir: String,
        #[arg(long, default_value = "default")]
        deffnm: String,
        #[arg(long,default_value = "default")]
        checkpoint_sub: String,
    },
    Add_Parent {
        #[arg(short='c', long, num_args = 1.., value_delimiter = ' ')] // space separated vector of all names of child directories to create monitors of 
        children: Vec<String>,  
        #[arg(short='g', long, default_value = "default")] // space separated vector of all child directories to create monitors of 
        group: String,  
        #[arg(short='n', long, default_value = "default")] // if left empty 
        name: String,
        #[arg(long, default_value = "cur_dir")]
        work_dir: String,
        #[arg(long, default_value = "default")]
        deffnm: String,
        #[arg(long,default_value = "default")]
        checkpoint_sub: String,
    },
    // delete monitor
    // takes flag: -n name
    Delete {
        name: String
    },
    // list monitors
    List{},
    // update a monitor provided its name
    // takes flag: -n name, if left empty will update all monitors WIP
    Update{ 
        #[arg(default_value = "all")]
        name: String 
    },
    Restart {
        name: String
    },
    Edit {
        name: String,
        feature: String,
        new_entry: String
    },
    Monitor {},
    New_Group {
        name: String,
    },
    Delete_Group {
        name: String
    },
    Add_To_Group {
        monitorname: String,
        groupname: String,
    },
}

fn main() {
    println!(" == gtop == ");

    // functionality
    // this enables cli function instead of tui function
    let args = Args::parse();
    match args.cmd{ // switch statement...
        Commands::Add { name, work_dir, deffnm, checkpoint_sub} => { // if create command...
            create_monitor(&name, &work_dir, &deffnm, &checkpoint_sub); // run create monitor func from create.rs
        }
        Commands::Add_Parent {children, group, name, work_dir, deffnm, checkpoint_sub} => {
            create_monitors_from_parent(children, &group, &name, &work_dir, &deffnm, &checkpoint_sub);
        }
        Commands::Delete { name } => { // if delete command...
            delete_monitor(&name); // run delete monitor func from delete.rs
        }
        Commands::List{} => { list_monitors(); } // if list command, run list monitors func from list.rs
        Commands::Update { name } => {
            let monitor : Vec<Monitor>;
            if name == "all" {
                monitor = return_all_monitors(); // return all monitors as a vector
            } else {
                monitor = vec![find_monitor(name)]; // find the monitor with the given name and return it as a vector
            }
            update_monitor(&monitor); // run update monitor func from get.rs
        }
        Commands::Restart { name } => { restart_monitor(name); }
        Commands::Edit { name, feature, new_entry } => { edit_monitor(name, feature, new_entry); }
        Commands::Monitor {} => { run_tui_monitor(); }
        Commands::New_Group {name} => { create_group(&name); }
        Commands::Delete_Group {name} => { delete_group(&name); }
        Commands::Add_To_Group {monitorname, groupname} => { add_monitor_to_group(&monitorname, &groupname); }
    }
}

fn run_tui_monitor() -> io::Result<()> {
       // tui
       let mut terminal = ratatui::init();
       let mut app: App = App { 
        exit: false,
        cur_screen: CurrentScreen::default(),
        cur_selection: 0,
        offset: 0,
        tick_rate: Duration::from_secs(5),
        last_tick: Instant::now(),
        cur_group_id: 4,
        cur_selected_monitor: "".to_string(),
        text_buffer: "".to_string(),
        keycode: "".to_string(),
    };
       let app_result = app.run(&mut terminal);

       ratatui::restore();
       app_result
}
