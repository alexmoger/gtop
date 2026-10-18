use clap::{Parser, Subcommand};
use std::io;
use std::time::{Duration, Instant};

mod monitors;
mod create;
mod get;
mod jobs;
mod app;
mod groups;
mod settings;

use create::{create_monitor, create_monitors_from_parent};
use crate::app::App;
use crate::app::CurrentScreen;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    // create monitor
    // args: -n MONITOR_NAME -p WORK_DIR -d DEFFNM -c CHECKPOINT_SUB
    // no required args, if run 'gtop create' itll instantiate the monitor with default values
    Create {
        #[arg(short='n', long, default_value = "default")]
        name: String,
        #[arg(long, short='p', default_value = "cur_dir")]
        work_dir: String,
        #[arg(long, short='d', default_value = "default")]
        deffnm: String,
        #[arg(long, short='c', default_value = "default")]
        checkpoint_sub: String,
    },
    // create parent 
    // args: -
    CreateParent {
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
}

fn main() {
    println!(" == gtop == ");

    // cli args handled with clap
    let args = Args::parse();
    match args.cmd{ 
        Some(Commands::Create { name, work_dir, deffnm, checkpoint_sub}) => {
            create_monitor(&name, &work_dir, &deffnm, &checkpoint_sub); // run create monitor func from create.rs
        }
        Some(Commands::CreateParent {children, group, name, work_dir, deffnm, checkpoint_sub}) => {
            create_monitors_from_parent(children, &group, &name, &work_dir, &deffnm, &checkpoint_sub);
        }
        None => { let _ = run_tui_monitor(); } // idk why the compiler told  me to do 'let _ =' 
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
