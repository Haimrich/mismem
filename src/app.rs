use std::mem;

use tui::widgets::TableState;


use crate::win::enum_windows;
use crate::win::enum_processes;

pub enum AppState {
    Home,
    SelectProcess,
    EditMemory,
}

pub struct App<> {
    pub state: AppState,
    pub table_state: TableState,
    pub items: Vec<Vec<String>>,
}

impl<> App<> {
    pub fn new() -> App<> {
        let mut app = App {
            state: AppState::SelectProcess,
            table_state: TableState::default(),
            items: vec![],
        };

        app.update();
        app
    }
    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }


    pub fn update(&mut self) {
        self.items.clear();
        
        for p in enum_processes() {
            self.items.push(vec![p.pid.to_string(), p.name, p.memory.to_string()]);
        }

        // for w in enum_windows() {
        //    self.items.push(vec![w.name, w.memory.to_string(), w.memory.to_string()]);
        //}
    }
}