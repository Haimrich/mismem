use tui::widgets::{TableState,ListState};
use tui_input::Input;

//use crate::win::enum_windows;
use crate::win::{enum_processes,check_process,scan_process,filter_process};
use crate::mem::{Memory,Datatype};


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

    // pub show_mismem_popup : bool,
    // pub mismem_input: Input,
}

impl<> App<> {
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
            self.memory.clear();
        } else {
            self.show_popup = true;
        }
        self.table_state.select(None);
    }


    pub fn search(&mut self) {
        // DATATYPE_OPTS = ["Byte", "2 Bytes","4 Bytes","8 Bytes","16 Bytes","Float","Double"];
        let (value_bytes, datatype) = match self.search_datatype.selected().unwrap_or(0) {
            0 => { // Byte
                match self.search_input.value().parse::<u8>() {
                    Ok(r_u8) => (r_u8.to_ne_bytes().to_vec(), Datatype::B1),
                    Err(_) => match self.search_input.value().parse::<i8>() {
                        Ok(r_i8) => (r_i8.to_ne_bytes().to_vec(), Datatype::B1S),
                        Err(e) => { 
                            self.popup_error = format!("Parsing error: {e}");
                            self.show_popup = true; 
                            return;
                        }
                    }
                }
            },
            1 => { // 2 Bytes
                match self.search_input.value().parse::<u16>() {
                    Ok(r_u16) => (r_u16.to_ne_bytes().to_vec(), Datatype::B2),
                    Err(_) => match self.search_input.value().parse::<i16>() {
                        Ok(r_i16) => (r_i16.to_ne_bytes().to_vec(), Datatype::B2S),
                        Err(e) => { 
                            self.popup_error = format!("Parsing error: {e}");
                            self.show_popup = true; 
                            return;
                        }
                    }
                }
            },
            2 => { // 4 Bytes
                match self.search_input.value().parse::<u32>() {
                    Ok(r_u32) => (r_u32.to_ne_bytes().to_vec(), Datatype::B4),
                    Err(_) => match self.search_input.value().parse::<i32>() {
                        Ok(r_i32) => (r_i32.to_ne_bytes().to_vec(), Datatype::B4S),
                        Err(e) => { 
                            self.popup_error = format!("Parsing error: {e}");
                            self.show_popup = true; 
                            return;
                        }
                    }
                }
            },
            3 => { // 8 Bytes
                match self.search_input.value().parse::<u64>() {
                    Ok(r_u64) => (r_u64.to_ne_bytes().to_vec(), Datatype::B8),
                    Err(_) => match self.search_input.value().parse::<i64>() {
                        Ok(r_i64) => (r_i64.to_ne_bytes().to_vec(), Datatype::B8S),
                        Err(e) => { 
                            self.popup_error = format!("Parsing error: {e}");
                            self.show_popup = true; 
                            return;
                        }
                    }
                }
            },
            4 => { // 16 Bytes
                match self.search_input.value().parse::<u128>() {
                    Ok(r_u128) => (r_u128.to_ne_bytes().to_vec(), Datatype::B16),
                    Err(_) => match self.search_input.value().parse::<i128>() {
                        Ok(r_i128) => (r_i128.to_ne_bytes().to_vec(), Datatype::B16S),
                        Err(e) => { 
                            self.popup_error = format!("Parsing error: {e}");
                            self.show_popup = true; 
                            return;
                        }
                    }
                }
            },
            5 => { // Float Bytes
                match self.search_input.value().parse::<f32>() {
                    Ok(r_f32) => (r_f32.to_ne_bytes().to_vec(), Datatype::F),
                    Err(e) => { 
                        self.popup_error = format!("Parsing error: {e}");
                        self.show_popup = true; 
                        return;
                    }
                }
            },
            6 => { // Double Bytes
                match self.search_input.value().parse::<f64>() {
                    Ok(r_f64) => (r_f64.to_ne_bytes().to_vec(), Datatype::D),
                    Err(e) => { 
                        self.popup_error = format!("Parsing error: {e}");
                        self.show_popup = true; 
                        return;
                    }
                }
            },
            _ => panic!("Illegal Value Type Option.")
        };

        self.edit_state = EditState::Busy;

        // SEARCH_MODE_OPTS = ["First Search", "Filter"];
        match self.search_mode.selected().unwrap_or(0) {
            0 => {
                self.memory = scan_process(self.selected_process, &value_bytes, &datatype, &mut self.search_progress);
            },
            1 => {
                filter_process(self.selected_process, &mut self.memory, &value_bytes, &datatype, &mut self.search_progress);
            },
            _ => {}
        }

        self.edit_state = EditState::Select;
    }

    // EditMem

    pub fn memory_next(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(0) + 1) % self.memory.len()
        ));
    }

    pub fn memory_previous(&mut self) {
        if self.show_popup { return; }

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

}