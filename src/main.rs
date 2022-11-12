mod app;
mod handler;
mod ui;
mod win;
mod mem;

use std::{sync::Arc, error::Error, io, time::{Instant, Duration}};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use tui::{
    backend::CrosstermBackend, Terminal
};

use app::App;
use handler::Handler;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(100);

    let app = Arc::new(tokio::sync::Mutex::new(App::new()));
    let app_ui = Arc::clone(&app);

    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);
    log::info!(" 😊 Hello!");

    tokio::spawn(async move {
        let mut handler = Handler::new(app);

        while let Some(event) = rx.recv().await {
            handler.handle(event).await;
        }
    });

    start_ui(&app_ui, &tx).await?;

    Ok(())
}

pub async fn start_ui(app: &Arc<tokio::sync::Mutex<App>>, tx: &tokio::sync::mpsc::Sender<Event>) -> io::Result<()> {
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(60);
    let mut last_tick = Instant::now();

    loop {        
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            let event = event::read()?;
            _ = tx.send(event).await;
        }

        if last_tick.elapsed() >= tick_rate {
            // app.on_tick();
            last_tick = Instant::now();
        }

        let mut app = app.lock().await;
        terminal.draw(|rect| ui::draw(rect, &mut app))?;

        if app.exiting {
            break
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
