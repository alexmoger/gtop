
// tui stuff
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    symbols,
    buffer::Buffer,
    layout::Rect,
    style::{Stylize, Style, Color, Modifier},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Gauge, BorderType, LineGauge},
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use std::io;
use std::time::{Duration, Instant};

use crate::monitors::Monitor;
use crate::aux::return_all_monitors;
use crate::aux::return_all_jobs;
use crate::jobs::{Job, JobFile, Status};
use crate::get::update_monitor;
use crate::delete::delete_monitor;

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    View1,
    View2,
    Delete,
    Restart,
    Add,
}

pub struct App {
    pub exit: bool,
    pub cur_screen: CurrentScreen,
    pub cur_selection: usize,
    pub offset: usize,
    pub tick_rate: Duration, // how often the tui allows an update
    pub last_tick: Instant, // last tick
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
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('r') => self.cur_screen = CurrentScreen::Restart,
                KeyCode::Char('d') => self.cur_screen = CurrentScreen::Delete,
                KeyCode::Char('a') => self.cur_screen = CurrentScreen::Add,
                KeyCode::Char('y') => {
                    match self.cur_screen {
                        CurrentScreen::Delete => {
                            let cur_monitor_name = &return_all_monitors()[self.cur_selection].name;
                            delete_monitor(cur_monitor_name);
                        },
                        _ => {}
                    }
                }
                KeyCode::Char('n') => {
                    match self.cur_screen {
                        CurrentScreen::Delete => { self.cur_screen = CurrentScreen::View1; },
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    match self.cur_screen {
                        CurrentScreen::View1 => { self.cur_screen = CurrentScreen::View2; },
                        CurrentScreen::View2 => self.cur_screen = CurrentScreen::View1,
                        _ => {}
                    }
                },
                KeyCode::Esc => { // clean this up 
                    match self.cur_screen { 
                        CurrentScreen::View2 => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Restart => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Delete => self.cur_screen = CurrentScreen::View1,
                        CurrentScreen::Add => self.cur_screen = CurrentScreen::View1,
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
                        _ => {}
                    }
                },
                KeyCode::Up => {
                    match self.cur_screen {
                        CurrentScreen::View1 => {
                            self.cur_selection = self.cur_selection.saturating_sub(1);  
                        },
                        _ => {}
                    }
                },
                _ => {},
            }
        }
        Ok(())
    }
}

// functions for RENDERING each screen defining the content
// MAIN SCREEN CONTIANING LIST OF MONITORS + BASIC INFO
fn screen_view1(area: Rect, buf: &mut Buffer, cur_sel : &usize, cur_offset: &usize) { // 'static defines the lifetime ie lives for the entire program duration
    let monitors: Vec<Monitor> = return_all_monitors(); // gets all monitors from monitor.toml
    let jobs: Vec<Job> = return_all_jobs();

    // RENDER STUFF
    let monitor_chunk_size: u16 = 5; // number of lines for each monitor
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
                Line::from(""),
                Line::from(status).bold(),
            ]))
            .block(block.clone())
            .render(*chunk, buf);
        } else {
            Paragraph::new(Text::from(vec![
                Line::from(monitor_names[i]).red(),
                //Line::from(monitor_pwd[i]).italic(),
                Line::from(""),
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
fn screen_view2(area: Rect, buf: &mut Buffer) { // 'static defines the lifetime ie lives for the entire program duration
    let block = Block::bordered() // frame around area
        .title("")
        .border_set(border::THICK);

    Paragraph::new(Text::from(vec![
                Line::from("WIP").bold().blue(),
                Line::from("NAME").bold().blue(),
                Line::from("PWD").bold(),
                Line::from("STATUS").bold(),
            ]))
            .block(block.clone())
            .centered()
            .render(area, buf);
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
        Line::from("Delete monitor? (y/n)"),
    ]))
        .centered()
        .render(area,buf);
}

// whats rendered from frame.render_widget...
impl Widget for &App { // uses reference since render doesnt edit anything
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where 
        Self: Sized,
    {
            // list of instructions on bottom of tui
            let instructions = Line::from(vec! [
                " Quit ".into(),
                "<q>".blue().bold(),
                " Select  ".into(),
                "<Enter>".blue().bold(),
                " Return ".into(),
                "<Esc>".blue().bold(),
                " Delete ".into(),
                "<d>".blue().bold(),
                " Restart ".into(),
                "<r> ".blue().bold(),
            ]);
            // sets frame around tui 
            let block = Block::bordered() // frame around area
                .title(Line::from(" == gtop == ").centered())
                .title_bottom(instructions.centered())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(160,66,184)))
                .border_set(border::THICK);
            let inner_area = block.inner(area);

            block.render(area,buf); // renders outer border

            // each function contains all the stuff to draw !!
            // takes in inner_area instead of area
            match self.cur_screen {
                CurrentScreen::View1 => screen_view1(inner_area, buf, &self.cur_selection, &self.offset),
                CurrentScreen::View2 => screen_view2(inner_area, buf),
                CurrentScreen::Restart => screen_restart(inner_area, buf, &self.cur_selection),
                CurrentScreen::Delete => screen_delete(inner_area, buf, &self.cur_selection),
                CurrentScreen::Add => screen_add(inner_area, buf),
            }
    }    
}
