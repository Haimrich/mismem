use std::sync::Arc;
use tokio::sync::Mutex;

use crossterm::event::{Event, KeyCode, MouseEventKind};
use tui_input::backend::crossterm::EventHandler;

use std::time::Instant;

use crate::{
    app::{App, AppState, EditState}, 
    win::{scan_process, filter_process, update_process, write_process}, 
    mem::Datatype
};

pub struct Handler {
    app: Arc<Mutex<App>>
}

impl Handler {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    
    pub async fn handle(&mut self, event: Event) {
        let mut app = self.app.lock().await;

        match event {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') { 
                    app.exiting = true 
                }
                
                match app.state {
                    AppState::SelectProcess => match key.code {
                        KeyCode::Down => app.next_process(),
                        KeyCode::Up => app.previous_process(),
                        KeyCode::Char('u') => app.update_process_list(),
                        KeyCode::Enter => app.select_process(),
                        _ => {}
                    }
                    AppState::EditMemory => match app.edit_state {
                    
                        EditState::Select => match key.code {
                            KeyCode::Down => app.next_memory(),
                            KeyCode::Up => app.previous_memory(),
                            KeyCode::Char('i') => app.input_mode(),
                            KeyCode::Char('s') => app.change_search_mode(),
                            KeyCode::Char('t') => app.change_search_datatype(),
                            KeyCode::Char('m') => app.change_search_type(),
                            KeyCode::Left | KeyCode::Esc => {
                                app.back()
                            },
                            KeyCode::Enter => {
                                app.select_memory()
                            },
                            KeyCode::Char('u') => {
                                drop(app);
                                self.update_memory().await;
                            },
                            _ => {}
                        },
                        EditState::Input => if app.show_popup { 
                            app.show_popup = false;
                        } else {
                            match key.code {
                                KeyCode::Enter => {
                                    drop(app);
                                    self.search().await;
                                },
                                KeyCode::Esc => {
                                    app.edit_state = EditState::Select;
                                },
                                _ => {
                                    app.search_input.handle_event(&Event::Key(key));
                                }
                            }
                        },
                        EditState::Edit => if app.show_popup { 
                            app.show_popup = false;
                        } else {
                            match key.code {
                                KeyCode::Enter => {
                                    drop(app);
                                    self.write().await;
                                },
                                KeyCode::Esc => {
                                    app.edit_state = EditState::Select;
                                },
                                _ => {
                                    app.mismem_input.handle_event(&Event::Key(key));
                                }
                            }
                        },
                        _ => {}
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse) => match app.state {
                AppState::SelectProcess => match mouse.kind {
                    MouseEventKind::ScrollUp => app.previous_process(),
                    MouseEventKind::ScrollDown => app.next_process(),
                    _ => {}
                },
                AppState::EditMemory => match mouse.kind {
                    MouseEventKind::ScrollUp => app.previous_memory(),
                    MouseEventKind::ScrollDown => app.next_memory(),
                    _ => {}
                },
                _ => {}
            }
            _ => {}
        }
    }

    async fn search(&mut self) {
        let mut app = self.app.lock().await;

        macro_rules! popup_error{
            ($e:expr)=>{{
                app.popup_error = format!("Parsing error: {}", $e);
                app.show_popup = true; 
                return;
            }}
        }

        macro_rules! parse{
            ($t:ty,$d:expr)=>{ 
                match app.search_input.value().parse::<$t>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d),
                    Err(e) => popup_error!(e)
                }
            };
            ($t1:ty,$d1:expr;$t2:ty,$d2:expr)=>{ 
                match app.search_input.value().parse::<$t1>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d1),
                    Err(_) => match app.search_input.value().parse::<$t2>() {
                        Ok(r) => (r.to_ne_bytes().to_vec(), $d2),
                        Err(e) => popup_error!(e)
                    }
                }
            }
        }

        // DATATYPE_OPTS = ["Byte", "2 Bytes","4 Bytes","8 Bytes","16 Bytes","Float","Double"];
        let (value_bytes, datatype) = match app.search_datatype.selected().unwrap_or(0) {
            0 => parse!(u8, Datatype::B1; i8, Datatype::B1S), // Byte
            1 => parse!(u16, Datatype::B2; i16, Datatype::B2S), // 2 Bytes
            2 => parse!(u32, Datatype::B4; i32, Datatype::B4S), // 4 Bytes,
            3 => parse!(u64, Datatype::B8; i64, Datatype::B8S), // 8 Bytes
            4 => parse!(u128, Datatype::B16; i128, Datatype::B16S), // 16 Bytes
            5 => parse!(f32, Datatype::F), // Float
            6 => parse!(f64, Datatype::D), // Double
            _ => panic!("Illegal Value Type Option.")
        };

        app.edit_state = EditState::Busy;

        let sel_proc = app.selected_process;
        
        // SEARCH_MODE_OPTS = ["First Search", "Filter"];
        let mode = app.search_mode.selected().unwrap_or(0);
        drop(app);

        match mode {
            0 => {
                scan_process(sel_proc, &value_bytes, &datatype, Arc::clone(&self.app)).await;
            },
            1 => {
                filter_process(sel_proc, &value_bytes, &datatype, Arc::clone(&self.app)).await;
            },
            _ => {}
        }

        let mut app = self.app.lock().await;
        app.edit_state = EditState::Select;
    }

    async fn update_memory(&mut self) {
        let mut app = self.app.lock().await;
        let now = Instant::now();
        app.edit_state = EditState::Busy;
        drop(app);

        update_process(Arc::clone(&self.app)).await;

        let elapsed = now.elapsed();
        log::info!(" Memory view updated in {:.3?} s.", elapsed.as_secs_f64());

        let mut app = self.app.lock().await;
        app.edit_state = EditState::Select;
    }

    async fn write(&mut self) {
        let mut app = self.app.lock().await;

        macro_rules! popup_error{
            ($e:expr)=>{{
                app.popup_error = format!("Parsing error: {}", $e);
                app.show_popup = true; 
                return;
            }}
        }

        macro_rules! parse{
            ($t:ty,$d:expr)=>{ 
                match app.mismem_input.value().parse::<$t>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d),
                    Err(e) => popup_error!(e)
                }
            };
        }

        let mut tokens = app.selected_address.split(':');
        let address = usize::from_str_radix(tokens.next().unwrap(), 16).unwrap();
        let (new_value_bytes, _)  = match tokens.next().unwrap() {
            "u8" => parse!(u8,Datatype::B1),
            "i8" => parse!(i8, Datatype::B1S),
            "u16" => parse!(u16, Datatype::B2),
            "i16" => parse!(i16, Datatype::B2S),
            "u32" => parse!(u32, Datatype::B4),
            "i32" => parse!(i32, Datatype::B4S),
            "u64" => parse!(u64, Datatype::B8),
            "i64" => parse!(i64, Datatype::B8S),
            "u128" => parse!(u128, Datatype::B16),
            "i128" => parse!(i128, Datatype::B16S),
            "f32" => parse!(f32, Datatype::F),
            "f64" => parse!(f64, Datatype::D),
            _ => panic!("Unrecognized type name"),
        };
        
        if !write_process(app.selected_process, address, &new_value_bytes) {
            app.popup_error = String::from("Error: can't write at target address.");
            app.show_popup = true; 
            log::error!(" Memory write failed.");
        } else {
            log::info!(" Memory write successful.");
        }

        app.edit_state = EditState::Busy;
        drop(app);

        update_process(Arc::clone(&self.app)).await;

        let mut app = self.app.lock().await;
        app.edit_state = EditState::Select;
    }

}
