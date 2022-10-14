mod app;
mod ui;
mod win;

use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend}, Terminal
};

use tui_input::backend::crossterm as input_backend;
use tui_input::backend::crossterm::EventHandler;

use app::{AppState,EditState};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = app::App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') { return Ok(()); }
            
            match app.state {
                AppState::SelectProcess => match key.code {
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Char('u') => app.update(),
                    KeyCode::Enter => app.select_process(),
                    _ => {}
                }
                AppState::EditMemory => match app.edit_state {
                
                    EditState::Select => match key.code {
                        KeyCode::Left | KeyCode::Esc => {
                            app.back()
                        },
                        KeyCode::Char('s') => app.change_search_mode(),
                        KeyCode::Char('t') => app.change_search_datatype(),
                        KeyCode::Char('m') => app.change_search_type(),
                        KeyCode::Char('i') => {
                            app.edit_state = EditState::Edit;
                        },
                        KeyCode::Down => app.memory_next(),
                        KeyCode::Up => app.memory_previous(),
                        _ => {}
                    }
                    EditState::Edit => match key.code {
                        KeyCode::Enter => {
                            app.search()
                        },
                        KeyCode::Esc => {
                            app.edit_state = EditState::Select;
                        },
                        _ => {
                            app.search_input.handle_event(&Event::Key(key));
                        }
                    }
                    _ => {}
                }
                _ => {}
            }
        }
    }
}


