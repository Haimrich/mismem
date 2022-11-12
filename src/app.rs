use tui::widgets::{TableState,ListState};
use tui_input::Input;

//use crate::win::enum_windows;
use crate::win::{enum_processes,check_process};
use crate::mem::Memory;


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
    pub selected_process: u32,

    pub search_input: Input,
    pub edit_state: EditState,
    pub memory: Memory,
    pub search_progress: f64,
    pub search_mode: ListState,
    pub search_datatype: ListState,
    pub search_type: ListState,

    pub show_popup: bool,
    pub popup_error : String,

    pub mismem_input: Input,
    pub selected_address: String,

    first_input: bool,
    pub exiting: bool,
}

impl App {
    pub const DATATYPE_OPTS : [&str;7] = ["Byte", "2 Bytes","4 Bytes","8 Bytes","16 Bytes","Float","Double"];
    pub const SEARCH_MODE_OPTS : [&str;2] = ["First Search", "Filter"];
    pub const MATCH_MODE_OPTS : [&str;3] = ["Exact Match", "Less Than", "Greater Than"];

    pub fn new() -> App<> {
        let mut app = App {
            state: AppState::SelectProcess,
            table_state: TableState::default(),
            processes: vec![],
            selected_process: 0,
            
            search_input: Input::from("Press i to input..."),
            edit_state: EditState::Select,
            memory: Memory::new(),
            search_progress: 0.0,
            search_mode: ListState::default(),
            search_datatype: ListState::default(),
            search_type: ListState::default(),

            show_popup: false,
            popup_error: String::new(),

            mismem_input: Input::default(),
            selected_address: String::new(),

            first_input: true,
            exiting: false,
        };

        app.search_mode.select(Some(0));
        app.search_datatype.select(Some(0));
        app.search_type.select(Some(0));

        app.update_process_list();
        app
    }

    pub fn next_process(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(self.processes.len() - 1) + 1) % self.processes.len()
        ));

    }

    pub fn previous_process(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.processes.len() + self.table_state.selected().unwrap_or(0) - 1) % self.processes.len()
        ));
    }

    pub fn update_process_list(&mut self) {
        if self.show_popup { return; }
        self.processes.clear();
        
        for p in enum_processes() {
            self.processes.push(vec![p.pid.to_string(), p.name, p.memory.to_string()]);
        }
    }

    pub fn back(&mut self) {
       match self.state {
            AppState::EditMemory => {
                self.show_popup = false;
                self.state = AppState::SelectProcess;
                self.table_state.select(None);
                self.update_process_list();
            }
            _ => {}
       }
    }

    pub fn select_process(&mut self) {

        if self.show_popup {
            self.show_popup = false;
            self.table_state.select(None);
            self.update_process_list();
            return;
        }

        if self.table_state.selected().is_none() { 
            return; 
        }

        self.selected_process = self.processes[self.table_state.selected().unwrap_or_default()][0].parse().unwrap();
        
        if check_process(self.selected_process) {
            self.state = AppState::EditMemory;
            self.memory.clear();
        } else {
            self.show_popup = true;
        }
        self.table_state.select(None);
    }


    pub fn select_memory(&mut self) {
        
        let memory_idx = self.table_state.selected().unwrap_or_default();
        let entry = self.memory.iter().nth(memory_idx);

        match entry {
            Some(entry) => {
                self.edit_state = EditState::Edit;
                self.mismem_input = Input::new(entry[1].clone());
                self.selected_address = entry[0].clone();
            }
            _ => {}
        }
    }

    // EditMem

    pub fn next_memory(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(self.memory.len() - 1) + 1) % self.memory.len()
        ));
    }

    pub fn previous_memory(&mut self) {
        if self.show_popup || self.table_state.selected().unwrap_or(0) == 0 { 
            return; 
        }
    
        self.table_state.select(Some(
            (self.memory.len() + self.table_state.selected().unwrap_or(0) - 1) % self.memory.len()
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

    pub fn input_mode(&mut self) {
        if self.first_input {
            self.first_input = false;
            self.search_input.reset();
        }
        self.edit_state = EditState::Input;
    }

}