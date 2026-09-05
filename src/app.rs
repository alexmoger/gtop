
// tui stuff
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    symbols,
    buffer::Buffer,
    layout::Rect,
    style::{Stylize, Style, Color, Modifier},
    symbols::border,
    text::{Line, Text, Span},
    widgets::{Block, Paragraph, Widget, Gauge, BorderType, LineGauge, Chart, Dataset, Axis, GraphType, Padding},
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use std::{fmt::format, io};
use std::time::{Duration, Instant};

use crate::{groups::{Group, delete_group}, monitors::Monitor};
use crate::monitors::{return_all_monitors, find_monitor, add_monitor_to_group};
use crate::jobs::{Job, JobFile, Status, find_job, return_all_jobs};
use crate::get::update_monitor;
use crate::delete::delete_monitor;
use crate::groups::{return_all_groups, return_monitors_in_group, rename_group, create_group};
use crate::settings::{load_settings, Settings};

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    View1,
    View2,
    Delete,
    Restart,
    Add,
    ChangeGroup,
    WriteText,
    Settings,
}

pub struct App {
    pub exit: bool,
    pub cur_screen: CurrentScreen,
    pub cur_selection: usize,
    pub offset: usize,
    pub tick_rate: Duration, // how often the tui allows an update
    pub last_tick: Instant, // last tick
    pub cur_group_id: usize, // group id 0 = all, 1 = unassigned, 2->N = groupsform groups.toml
    pub cur_selected_monitor: String, // name of monitor selected after pressing 'g', for changing group stuff SO FAR IDK CAN USE FOR OTHER STUFF PROBABLY
    pub text_buffer: String, 
    pub keycode: String,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit { // main app loop
            let timeout = self.tick_rate
                .checked_sub(self.last_tick.elapsed()) // returns tick rate - how much time has passed since last tick
                .unwrap_or(Duration::from_secs(0)); // returns 0 if tick rate > last tick

            // returns true if something pressed or false if timeout passes without no input
            if crossterm::event::poll(timeout)? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                    _ => {}
                }
            }

            // if tick rate has passed, run all the ontick functions
            if self.last_tick.elapsed() >= self.tick_rate {
                self.on_tick();
                self.last_tick = Instant::now();
            }
            
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn on_tick(&mut self) {
        let monitors = return_all_monitors();
        update_monitor(&monitors);
    }

    // RENDERS STUFF (all in impl Widget)sa
    fn draw(&mut self, frame: &mut Frame) {
        self.update_offset(frame.area());
        frame.render_widget(&*self, frame.area()); // renders impl Widget
    }

    // for view 1 monitor scrolling, gets the offset when n monitors > monitors displayed
    fn update_offset(&mut self, area: Rect) {
        let monitors = return_all_monitors();
        let n_monitors = monitors.len();
        let monitor_chunk_size: u16 = 5; // number of lines for each monitor
        let max_monitors_on_screen = (area.height / monitor_chunk_size) as usize; // gets max number of chunks that can be displayed
        let n_monitors_to_display = n_monitors.min(max_monitors_on_screen);

        if self.cur_selection >= n_monitors { // clamps 
            self.cur_selection = n_monitors.saturating_sub(1);
        }
        if self.cur_selection < self.offset { // if selected < first monitor displayed, moved first monitor displayed back
            self.offset = self.cur_selection;
        } else if n_monitors_to_display > 0 && self.cur_selection >= self.offset + n_monitors_to_display { // if selection goes above  monitors allowed to display
            self.offset = self.cur_selection + 1 - n_monitors_to_display; // set offset 
        }
        self.offset = self.offset.min(n_monitors.saturating_sub(n_monitors_to_display));
    }

    // KEY PRESS STUFF
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
        if key_event.kind == KeyEventKind::Press {
            if let CurrentScreen::WriteText = self.cur_screen { // FOR WRITING  
                match key_event.code {
                    KeyCode::Char(c) => { 
                        if self.text_buffer.len() < 20 { 
                            self.text_buffer.push(c); 
                        }
                    }, // write characters c
                    KeyCode::Backspace => { self.text_buffer.pop(); },
                    KeyCode::Enter => {
                        match self.keycode.as_str() {
                            "n" => { // n = changing group name 
                                self.cur_screen = CurrentScreen::View1;
                                rename_group(&self.cur_group_id, &self.text_buffer);
                            },
                            "ng" => { // create new group
                                self.cur_screen = CurrentScreen::View1;
                                create_group(&self.text_buffer);
                            }
                            _ => { panic!("howtf"); },
                        }
                    },
                    KeyCode::Esc => {
                        self.text_buffer.clear();
                        self.cur_screen = CurrentScreen::View1;
                    },
                    _ => {},
                }
                return Ok(()); // stops next match statement from occuring ie. blocks all normal shortcut stuff while writing 
            }
            match key_event.code { // else handle all normal shortcut stuff 
                KeyCode::Char('q') => {
                    match self.cur_screen {
                        CurrentScreen::View1 => self.exit = true,
                        _ => {}
                    }   
                }    
                //KeyCode::Char('r') => self.cur_screen = CurrentScreen::Restart,
                KeyCode::Char('d') => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            if self.cur_group_id > 4 {
                                self.cur_screen = CurrentScreen::Delete;
                                self.keycode = "dg".to_string();
                            }
                        },
                        CurrentScreen::View2 => { self.cur_screen = CurrentScreen::Delete; self.keycode = "dm".to_string(); },
                        _ => {},
                    }   
                }    
                //KeyCode::Char('a') => self.cur_screen = CurrentScreen::Add,
                KeyCode::Char('y') => {
                    match self.cur_screen {
                        CurrentScreen::Delete => {
                            let kc : &str = &self.keycode;
                            match kc {
                                "dg" => {
                                    let real_group_id = self.cur_group_id - 5;
                                    let cur_group_name = return_all_groups()[real_group_id].name.clone();
                                    return_monitors_in_group(&self.cur_group_id).iter().for_each(|m| add_monitor_to_group(&m.name, "")); // UNASSIGNS MONITORS IN DELETED GROUP
                                    delete_group(&cur_group_name);
                                    self.cur_screen = CurrentScreen::View1;
                                    self.cur_group_id = 4;
                                },
                                "dm" => {
                                    let cur_monitor_name = &return_all_monitors()[self.cur_selection].name;
                                    delete_monitor(cur_monitor_name);
                                    self.cur_screen = CurrentScreen::View1; // resets the screen 
                                },
                                _ => {panic!("AAAAAAAAAAAAAAA")},
                            }                            
                        },
                        _ => {}
                    }
                }
                KeyCode::Char('n') => {
                    match self.cur_screen {  
                        CurrentScreen::View1 => {
                            self.cur_screen = CurrentScreen::WriteText;
                            self.text_buffer = "".to_string();
                            self.keycode = "ng".to_string();
                        },                   
                        CurrentScreen::Delete => { self.cur_screen = CurrentScreen::View1; },
                        CurrentScreen::ChangeGroup => { self.cur_screen = CurrentScreen::View1; self.cur_selection = 0; },
                        _ => {}
                    }
                }
                KeyCode::Char('s') => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            self.cur_screen = CurrentScreen::Settings; 
                            self.cur_selection = 0;
                        },
                        _ => {}
                    }
                }
                KeyCode::Char('g') => {
                    match self.cur_screen {
                        CurrentScreen::View1 => { 
                            if self.cur_group_id > 4 { // if on custom groups
                                self.cur_screen = CurrentScreen::WriteText;
                                self.text_buffer = "".to_string();  
                                self.keycode = "n".to_string(); 
                            }
                        },
                        CurrentScreen::View2 => {
                            self.cur_screen = CurrentScreen::ChangeGroup; // change screen
                            let monitors_in_cur_group = return_monitors_in_group(&self.cur_group_id); // gets all monitors in currently selected group
                            self.cur_selected_monitor = monitors_in_cur_group[self.cur_selection].name.clone(); // gets name of monitor selected on cur group screen
                            self.cur_selection = 0; 
                        },
                        _ => {}
                    }
                }
                KeyCode::Char('u') => { 
                    match self.cur_screen {
                        CurrentScreen::ChangeGroup => { // unassigns group from selection
                            add_monitor_to_group(&self.cur_selected_monitor, "");
                            self.cur_screen = CurrentScreen::View1; // reset screen
                            self.cur_selection = 0; // reset selection
                        },
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            self.cur_screen = CurrentScreen::View2;
                            let monitors_in_cur_group = return_monitors_in_group(&self.cur_group_id); // gets all monitors in currently selected group
                            self.cur_selected_monitor = monitors_in_cur_group[self.cur_selection].name.clone(); // gets name of monitor selected on cur group screen                       
                        }
                        CurrentScreen::ChangeGroup => {
                            add_monitor_to_group(&self.cur_selected_monitor, &return_all_groups()[self.cur_selection].name.clone());
                            self.cur_screen = CurrentScreen::View1; // reset screen
                            self.cur_selection = 0; // reset selection
                        },
                        CurrentScreen::WriteText => {
                            self.cur_screen = CurrentScreen::View1;
                        }
                        _ => {}
                    }
                },
                KeyCode::Esc => { // clean this up 
                    match self.cur_screen { 
                        CurrentScreen::View2 => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Restart => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Delete => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Add => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::ChangeGroup => {self.cur_screen = CurrentScreen::View1; self.cur_selection = 0; },
                        CurrentScreen::Settings => self.cur_screen = CurrentScreen::View1,
                        _ => {}
                    }
                },
                KeyCode::Down => { // edit this so that cant go above max n monitors (currently ill incease to infiinity)
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            let n_monitors = return_all_monitors().len(); // prob shit but idk 
                            if self.cur_selection + 1 < n_monitors {
                                self.cur_selection += 1; // edit this with another match so you cant change cur sel while not in view 1 or view 2 
                            }  
                        },
                        CurrentScreen::ChangeGroup => {
                            let n_groups = return_all_groups().len();
                            if self.cur_selection + 1 < n_groups {
                                self.cur_selection += 1;
                            }
                        },
                        CurrentScreen::Settings => {
                            let n_settings = 3; // MAX NUMBER OF SETTINGS THIS NEEDS TO BE A CONSTANT OR SOMEHTING ELSE )
                            if self.cur_selection + 1 < n_settings {
                                self.cur_selection += 1;
                            }
                        }
                        _ => {}
                    }
                },
                KeyCode::Up => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            self.cur_selection = self.cur_selection.saturating_sub(1);  
                        },
                        CurrentScreen::ChangeGroup => {
                            self.cur_selection = self.cur_selection.saturating_sub(1);
                        },
                        CurrentScreen::Settings => {
                            self.cur_selection = self.cur_selection.saturating_sub(1);
                        }
                        _ => {}
                    }
                },
                KeyCode::Right => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            let n_groups = return_all_groups().len(); // get number of groups in groups.toml also prob shit
                            if self.cur_group_id + 1 < ( n_groups + 5 ) { // n groups + 2 since 0 -> 4 is taken by preset groups 
                                self.cur_group_id += 1;
                            }
                            self.cur_selection = 0;
                        },
                        _ => {}
                    }
                }
                KeyCode::Left => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            self.cur_group_id = self.cur_group_id.saturating_sub(1);
                            self.cur_selection = 0;
                        },
                        _ => {}
                    }
                }
                _ => {},
            }
        }
        Ok(())
    }
}

// MAIN SCREEN CONTIANING LIST OF MONITORS + BASIC INFO
fn screen_view1(area: Rect, buf: &mut Buffer, cur_sel : &usize, cur_offset: &usize, cur_group_id: &usize) { // 'static defines the lifetime ie lives for the entire program duration
    let monitors: Vec<Monitor> = return_all_monitors(); // gets all monitors from monitor.toml
    let jobs: Vec<Job> = return_all_jobs();
    // this stuff is for etting all monitors with the current group selection
    let monitors_in_group = return_monitors_in_group(cur_group_id);
    let jobs_in_group : Vec<Job> = monitors_in_group // gt all jobs corresponding to monitors in gorup
        .iter()
        .map(|m| find_job(m.name.clone()))
        .collect();
    let monitors = monitors_in_group; // redefine monitors here 
    let jobs = jobs_in_group; //redefine jobs here    

    // RENDER STUFF
    let monitor_chunk_size: u16 = 4; // number of lines for each monitor
    let max_monitors_on_screen = (area.height / monitor_chunk_size) as usize; // gets max number of chunks that can be displayed
    let n_monitors = monitors.len(); // gets number of monitors 
    let n_monitors_to_display = if n_monitors < max_monitors_on_screen  // set number of monitors rendered as nmonitors or nmaxmonitorsthat can be rendered
    {
        n_monitors
    } else {
        max_monitors_on_screen
    }; // sets how many monitors are displayed, only so many are allowed (height/lines per monitor)


    let first_monitor_to_render: usize = *cur_offset;
    let last_monitor_to_render = (first_monitor_to_render + (n_monitors_to_display)).min(n_monitors);
    let monitors_to_render: &[Monitor] = &monitors[(first_monitor_to_render as usize)..last_monitor_to_render];
    let jobs_to_render: &[Job] = &jobs[(first_monitor_to_render as usize)..last_monitor_to_render];

    // CONTENT STUFF
    // maybe a cleaner way of doing this but it works !
    let monitor_names: Vec<&str> = monitors_to_render // gets a vec of all monitor names 
        .iter()    
        .map(|m| m.name.as_str())
        .collect();
    let monitor_status: Vec<Status> = jobs_to_render  // vec of all status
        .iter()
        .map(|j| j.status.clone())
        .collect();
    let monitor_curstep: Vec<String> = jobs_to_render // vec of all cur step
        .iter()
        .map(|j| j.cur_step.clone())
        .collect();
    let monitor_maxstep: Vec<String> = jobs_to_render // vec of all max step
        .iter()
        .map(|j| j.max_step.clone())
        .collect();
    let monitor_dt: Vec<String> = jobs_to_render
        .iter()
        .map(|j| j.time_step.clone())
        .collect();
    let monitor_pwd: Vec<&str> = monitors_to_render // i have no fking idea why changing all monitor stff to &str instead of String works for the print stuff ......
        .iter()
        .map(|m| m.work_dir.as_str())
        .collect();


    // creates number of constraints for each monitor chunk (y size)
    let constraints = (0..n_monitors_to_display)  // iterates between 0->nchunks
        .map(|_| Constraint::Length(monitor_chunk_size))
        .collect::<Vec<_>>(); // collects a vec of constraint lengths
    // creates each monitor chunk
    let monitor_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.clone())
        .split(area); 
    // creates each progress bar chunk   
    let split_screen_horizontal_chunks = Layout::default() // splits the screen into two segments horizontally by 50% ( FOR PROGRESS BAR, 1ST PART)
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    let progressbar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.clone())
        .split(split_screen_horizontal_chunks[1]); //  0 = left chunk 1 = right chunk

    // border for monitor chunks
    let block = Block::bordered() // frame around area
        .title("")
        .border_set(border::THICK);

    // renders all chunks -- content of each chunk in the paragraph
    for (i, chunk) in monitor_chunks.iter().enumerate() {
        let status = match monitor_status[i] {
            Status::Finished => "FINISHED",
            Status::Paused => "PAUSED",
            Status::Running => "RUNNING",
            Status::Waiting => "WAITING",
        };
        //stuff for progress bar, completetion percent and cur and max time
        // check if curstep[i] is a value (running, stopped, paused) or doesnt exist (0, waiting) 
        let graph_prog_bar : bool = match monitor_curstep[i].parse::<f32>() { // attempt to parse cur_step as a float... 
            Ok(_) => true, // if true, graph prog bar 
            Err(_) => false, // if error, dont !
        };
        
        let completion_percentage: u16;
        let cur_time: f32;
        let max_time: f32;
        // graph prog bar
        if graph_prog_bar {
            completion_percentage = (monitor_curstep[i].parse::<f32>().unwrap() / monitor_maxstep[i].parse::<f32>().unwrap() * 100.0) as u16;
            cur_time = (monitor_curstep[i].parse::<f32>().unwrap() * monitor_dt[i].parse::<f32>().unwrap()) / 1000000.0; // divide by 1mil to get microseconds
            max_time = (monitor_maxstep[i].parse::<f32>().unwrap() * monitor_dt[i].parse::<f32>().unwrap()) / 1000000.0;
        } else {
            completion_percentage = 0 as u16;
            cur_time = 0 as f32; 
            max_time = 0 as f32;
        }

        // draws each monitor 
        if first_monitor_to_render + i == *cur_sel as usize { // change this to just change the colour x)
            Paragraph::new(Text::from(vec![
                Line::from(monitor_names[i]).bold().blue(),
                //Line::from(monitor_pwd[i]).bold(),
                //Line::from(""),
                Line::from(status).bold(),
            ]))
            .block(block.clone())
            .render(*chunk, buf);
        } else {
            Paragraph::new(Text::from(vec![
                Line::from(monitor_names[i]).red(),
                //Line::from(monitor_pwd[i]).italic(),
                //Line::from(""),
                Line::from(status),
            ]))
            .block(block.clone())
            .render(*chunk, buf);
        }
        // progress bar
        /*
        Gauge::default() 
            .block(block.clone())
            .gauge_style(Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD)
            )
            .percent(completion_percentage)
            .label(format!("{:.2} / {:.2}", cur_time, max_time))
            .use_unicode(true) // does fk all
            .render(progressbar_chunks[i], buf);
        */
        LineGauge::default()
            .block(block.clone())
            .filled_style(Style::default().fg(Color::Magenta))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .filled_symbol("=") // or NORMAL, DOUBLE, ROUNDED
            .unfilled_symbol("-")
            .ratio(completion_percentage as f64 / 100.0)
            .label(format!("{:.2} / {:.2}", cur_time, max_time))
            .render(progressbar_chunks[i], buf);
    }
}
fn screen_view2(area: Rect, buf: &mut Buffer, cur_monitor_name: &str) { // 'static defines the lifetime ie lives for the entire program duration
    // get stuff to display
    let monitor = find_monitor(cur_monitor_name.to_string());
    let job = find_job(cur_monitor_name.to_string());
    let monitor_status = match job.status {
            Status::Finished => "FINISHED",
            Status::Paused => "PAUSED",
            Status::Running => "RUNNING",
            Status::Waiting => "WAITING",
    };

    // get chunks for splitting terminal
    let top_chunk_size = 4; // number of lines for top most chunk spanning whole terminal horizontally
    let vertical_chunks = Layout::default() // split terminal into a top and bottom chunk
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_chunk_size),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(vertical_chunks[1]); // 0 = top 1 = bottom
    let top_chunk = vertical_chunks[0];
    let bottom_left_chunk = horizontal_chunks[0];
    let bottom_right_chunk = horizontal_chunks[1];
    
    // chunk border
    let block = Block::bordered() // frame around area
        .title("")
        .border_set(border::THICK);

    //          top chunk
    Paragraph::new(Text::from(vec![
                Line::from(cur_monitor_name).bold().blue(),
                Line::from(monitor.group_name),
            ]))
            .block(block.clone())
            .centered()
            .render(top_chunk, buf);
    //          bottom left chunk
    Paragraph::new(Text::from(vec![
                Line::from(format!("status: {}", monitor_status)).centered(),
                Line::from(format!("pwd: {}", monitor.work_dir)).centered(),
                Line::from(format!("deffnm: {}", monitor.deffnm)).centered(),
                Line::from(format!("dir size: wip",)).centered(),
                Line::from(format!("cpi: {}", monitor.checkpoint_sub)).centered(),
            ]))
            .block(block.clone())
            .centered()
            .render(bottom_left_chunk, buf);
    //          bottom right chunk
    let data: &[(f64, f64)]  = &[(0.0,0.0), (1.0,2.0), (2.0,3.0), (3.0,2.0), (4.0,5.0), (5.0,5.0)]; // wip temp data
    let data_to_plot= Dataset::default()
        .name("")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Magenta))
        .data(data);
    let chart = Chart::new(vec![data_to_plot])
        .block(block.clone().padding(Padding::uniform(2)))
        .x_axis(
            Axis::default()
            .title("NSTEPS")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0,5.0])
            .labels(vec![
                Span::raw(format!("{:.0}", 0)),
                Span::raw(format!("{:.0}", 5)),
            ]),
        )
        .y_axis(
            Axis::default()
            .title("TIME_HOURS")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0,5.0])
            .labels(vec![
                Span::raw(format!("{:.0}", 0)),
                Span::raw(format!("{:.0}", 5)),
            ]),
        );
    chart.render(bottom_right_chunk, buf);
}
fn screen_restart(area: Rect, buf: &mut Buffer, cur_sel: &usize) { // 'static defines the lifetime ie lives for the entire program duration
        let monitors = return_all_monitors();
    // get cur sel monitor
    let cur_monitor_index = *cur_sel;
    let cur_sel_monitor = &monitors[cur_monitor_index];
    // display cur monitor name and status and complete percent
    Paragraph::new(Text::from(vec![
        Line::from("Currently selected:").bold(),
        Line::from(cur_sel_monitor.name.clone()),
        Line::from(" "),
        Line::from(" "),
        Line::from(" "),
        Line::from(" "),
        Line::from("Restart monitor? (y/n)"),
    ]))
        .centered()
        .render(area,buf);
}
fn screen_add(area: Rect, buf: &mut Buffer) { // 'static defines the lifetime ie lives for the entire program duration
    Paragraph::new("Add")
        .centered()
        .render(area,buf)
}
fn screen_delete(area: Rect, buf: &mut Buffer, cur_sel: &usize) { // displays the delete screen for the selected monitor -- functionality handled in key press event above
    //let monitors = return_all_monitors();
    // get cur sel monitor
    //let cur_monitor_index = *cur_sel;
    //let cur_sel_monitor = &monitors[cur_monitor_index];
    // display cur monitor name and status and complete percent
    Paragraph::new(Text::from(vec![
        Line::from("Delete Current Selection? (y/n)").red().bold(),
    ]))
        .centered()
        .render(area,buf);
}
fn screen_changegroup(area:Rect, buf: &mut Buffer, cur_sel: &usize, cur_offset: &usize) { // displays a list of all groups on toml, select group + reassign with enter
    let groups = return_all_groups(); // gets a vec of all groups in groups.toml
    
    // get groups to render 
    let group_chunk_size: u16 = 3; // size of each displayed group in vertical lines
    let max_groups_on_screen: usize = (area.height / group_chunk_size) as usize; // get max n groups to display 
    let n_groups = groups.len(); // get number of groups in groups.toml
    let n_groups_to_display = if n_groups < max_groups_on_screen { n_groups } else {max_groups_on_screen }; // sets groups to display on screen
    let first_group_to_render: usize = *cur_offset;
    let last_group_to_render: usize = (first_group_to_render + (n_groups_to_display)).min(n_groups);
    let groups_to_render: &[Group] = &groups[(first_group_to_render as usize)..last_group_to_render];
    // render stuff
    let constraints = (0..n_groups_to_display) // creates constraints for each displayed group y size
        .map(|_| Constraint::Length(group_chunk_size))
        .collect::<Vec<_>>();
    let group_chunks = Layout::default() // creates vertical chunks for each rendered group
        .direction(Direction::Vertical)
        .constraints(constraints.clone())
        .split(area);
    let block = Block::bordered() // for frame around each group
        .title("")
        .border_set(border::EMPTY);

    // gets content to add to screen
    let group_names: Vec<&str> = groups_to_render // list of group names
        .iter()
        .map(|g| g.name.as_str())
        .collect();
    // << TO ADD >>
    // get number of monitors in each group

    // !! RENDER !!
    for (i, chunk) in group_chunks.iter().enumerate() {
        if first_group_to_render + i == *cur_sel as usize { // renders currently selected group
            Paragraph::new(Text::from(vec![
                Line::from(group_names[i]).bold().blue(),
            ]))
            .block(block.clone())
            .centered()
            .render(*chunk, buf);
        } else { // renders all unselected groups
            Paragraph::new(Text::from(vec![
                Line::from(group_names[i]).bold().red(),
            ]))
            .block(block.clone())
            .centered()
            .render(*chunk, buf);
        }
    }
}
fn screen_writetext(area:Rect, buf:&mut Buffer, text: &str) {
    let block = Block::bordered()
        .title(Line::from("  Enter Text (max char 20)  ").centered())
        .border_set(border::THICK);

    Paragraph::new(Text::from(vec![
        Line::from(text).bold().centered(),
    ]))
    .block(block)
    .render(area, buf);
}
fn screen_settings(area:Rect, buf:&mut Buffer, cur_sel: &usize) {
    let settings = load_settings(); // gets a vec of all groups in groups.toml
    let setting_names = ["default deffnm", "default checkpoint submission script", "default monitor name"];

    let constraints = (0..setting_names.len()) // creates constraints for each displayed group y size
        .map(|_| Constraint::Length(3))
        .collect::<Vec<_>>();
    let chunks = Layout::default() // creates vertical chunks for each rendered group
        .direction(Direction::Vertical)
        .constraints(constraints.clone())
        .split(area);

    let block = Block::bordered() // for frame around border not each setting 
        .title("")
        .border_set(border::THICK)
        .padding(Padding::uniform(1));

    for i in 0..setting_names.len() {
        if &i == cur_sel {
            Paragraph::new(Text::from(vec![
                Line::from(setting_names[i]).bold().blue(),
            ]))
            //.block(block.clone())
            .centered()
            .render(chunks[i], buf);
        } else {
            Paragraph::new(Text::from(vec![
                Line::from(setting_names[i]).bold(),
            ]))
            //.block(block.clone())
            .centered()
            .render(chunks[i], buf);
        }
    }
}

// whats rendered from frame.render_widget...
impl Widget for &App { // uses reference since render doesnt edit anything
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where 
        Self: Sized,
    {
            let view1_instructions = vec! [
                " Quit ".into(),
                "<q>".red().bold(),
                " Select ".into(),
                "<Enter>".green().bold(),
                " New Group ".into(),
                "<n>".blue().bold(),
                " Rename Group ".into(),
                "<g>".blue().bold(),
                " Delete Group ".into(),
                "<d>".blue().bold(),
                " Settings ".into(),
                "<s> ".blue().bold(),
            ];
            let view2_instructions = vec![
                " Return ".into(),
                "<Esc>".red().bold(),
                " Delete ".into(),
                "<d>".blue().bold(),
                " Change Group ".into(),
                "<g>".blue().bold(),
            ];
            let delete_instructions = vec! [
                " Return ".into(),
                "<Exit>".red().bold(),
            ];
            let changegroup_instructions = vec! [
                " Select  ".into(),
                "<Enter>".green().bold(),
                " Return ".into(),
                "<Esc>".red().bold(),
                " Unassign Group ".into(),
                "<u>".blue().bold(),
            ];
            let writetext_instructions = vec! [
                " Enter  ".into(),
                "<Enter>".green().bold(),
                " Return ".into(),
                "<Esc> ".red().bold(),
            ];
            let settings_instructions = vec! [
                " Select ".into(),
                "<Enter>".green().bold(),
                " Return ".into(),
                "<Esc> ".red().bold(),
            ];
            let instructions = match self.cur_screen {
                CurrentScreen::View1 => Line::from(view1_instructions),
                CurrentScreen::View2 => Line::from(view2_instructions),
                CurrentScreen::Delete => Line::from(delete_instructions),
                CurrentScreen::ChangeGroup => Line::from(changegroup_instructions),
                CurrentScreen::WriteText => Line::from(writetext_instructions),
                CurrentScreen::Settings => Line::from(settings_instructions),
                _ => Line::from(view1_instructions)
            };

            let groups = return_all_groups();
            let mut group_name = "";
            match self.cur_group_id {
                0 => group_name = "Finished",
                1 => group_name = "Paused", 
                2 => group_name = "Running",
                3 => group_name = "Unassigned", // unassigned groups have the name ""
                4 => group_name = "All", 
                _ => group_name = &groups[self.cur_group_id - 5].name,
            }
            // sets frame around tui 
            let block = Block::bordered() // frame around area
                .title(Line::from(" == gtop == ").centered())
                .title(Line::from(format!(" < Group {} > ",  group_name)).left_aligned())
                .title_bottom(instructions.centered())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(160,66,184)))
                .border_set(border::THICK);
            let inner_area = block.inner(area);
            block.render(area,buf); // renders outer border

            // each function contains all the stuff to draw !!
            // takes in inner_area instead of area
            match self.cur_screen {
                CurrentScreen::View1 => screen_view1(inner_area, buf, &self.cur_selection, &self.offset, &self.cur_group_id),
                CurrentScreen::View2 => screen_view2(inner_area, buf, &self.cur_selected_monitor),
                CurrentScreen::Restart => screen_restart(inner_area, buf, &self.cur_selection),
                CurrentScreen::Delete => screen_delete(inner_area, buf, &self.cur_selection),
                CurrentScreen::Add => screen_add(inner_area, buf),
                CurrentScreen::ChangeGroup => screen_changegroup(inner_area, buf, &self.cur_selection, &self.offset),
                CurrentScreen::WriteText => screen_writetext(inner_area, buf, &self.text_buffer),
                CurrentScreen::Settings => screen_settings(inner_area, buf, &self.cur_selection),
            }
    }    
}
