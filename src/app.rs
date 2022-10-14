use tui::widgets::{TableState,ListState};
use tui_input::Input;

//use crate::win::enum_windows;
use crate::win::{enum_processes,check_process,scan_process};


pub enum AppState {
    Home,
    SelectProcess,
    EditMemory,
}

pub enum EditState {
    Input,
    Select,
    Edit,
    Busy,
}

pub struct App<> {
    pub state: AppState,
    pub table_state: TableState,
    pub processes: Vec<Vec<String>>,
    pub show_popup: bool,
    pub selected_process: u32,

    pub search_input: Input,
    pub edit_state: EditState,
    pub memory_entries: Vec<[i64;3]>,
    pub search_progress: f64,
    pub search_mode: ListState,
    pub search_datatype: ListState,
    pub search_type: ListState,
}

impl<> App<> {
    pub const DATATYPE_OPTS : [&str;6] = ["Byte", "2 Byte","4 Byte","8 Byte","Float","Double"];
    pub const SEARCH_MODE_OPTS : [&str;2] = ["First Search", "Filter"];
    pub const MATCH_MODE_OPTS : [&str;3] = ["Exact Match", "Less Than", "Greater Than"];

    pub fn new() -> App<> {
        let mut app = App {
            state: AppState::SelectProcess,
            table_state: TableState::default(),
            processes: vec![],
            show_popup: false,
            selected_process: 0,

            search_input: Input::from("Press i to input..."),
            edit_state: EditState::Select,
            memory_entries: vec![],
            search_progress: 0.0,
            search_mode: ListState::default(),
            search_datatype: ListState::default(),
            search_type: ListState::default(),
        };

        app.search_mode.select(Some(0));
        app.search_datatype.select(Some(0));
        app.search_type.select(Some(0));

        app.update();
        app
    }

    pub fn next(&mut self) {
        if self.show_popup { return; }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.processes.len() - 1 {
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
        if self.show_popup { return; }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.processes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn update(&mut self) {
        if self.show_popup { return; }
        self.processes.clear();
        
        for p in enum_processes() {
            self.processes.push(vec![p.pid.to_string(), p.name, p.memory.to_string()]);
        }

        // for w in enum_windows() {
        //    self.items.push(vec![w.name, w.memory.to_string(), w.memory.to_string()]);
        //}
    }

    pub fn back(&mut self) {
       match self.state {
            AppState::EditMemory => {
                self.show_popup = false;
                self.state = AppState::SelectProcess;
                self.table_state.select(None);
                self.update();
            }
            _ => {}
       }
    }

    pub fn select_process(&mut self) {

        if self.show_popup {
            self.show_popup = false;
            self.table_state.select(None);
            self.update();
            return;
        }

        if self.table_state.selected().is_none() { 
            return; 
        }

        self.selected_process = self.processes[self.table_state.selected().unwrap_or_default()][0].parse().unwrap();
        
        if check_process(self.selected_process) {
            self.state = AppState::EditMemory;
            self.memory_entries.clear();
        } else {
            self.show_popup = true;
        }
        self.table_state.select(None);
    }

    pub fn search(&mut self) {
        self.edit_state = EditState::Busy;
        let value = self.search_input.value().parse::<i64>().unwrap_or_default();
        let value_bytes = value.to_ne_bytes();

        self.memory_entries = scan_process(self.selected_process, value, &value_bytes, &mut self.search_progress);
        self.edit_state = EditState::Select;
    }

    // EditMem

    pub fn memory_next(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(0) + 1) % self.memory_entries.len()
        ));
    }

    pub fn memory_previous(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.memory_entries.len() + self.table_state.selected().unwrap_or(0) - 1) % self.memory_entries.len()
        ));
    }

    pub fn change_search_mode(&mut self) {
        self.search_mode.select(Some(
            (self.search_mode.selected().unwrap_or(0) + 1) % App::SEARCH_MODE_OPTS.len()
        ));
    }

    pub fn change_search_datatype(&mut self) {
        self.search_datatype.select(Some(
            (self.search_datatype.selected().unwrap_or(0) + 1) % App::DATATYPE_OPTS.len()
        ));
    }

    pub fn change_search_type(&mut self) {
        self.search_type.select(Some(
            (self.search_type.selected().unwrap_or(0) + 1) % App::MATCH_MODE_OPTS.len()
        ));
    }

}