use tui::widgets::{TableState,ListState};
use tui_input::Input;

//use crate::win::enum_windows;
use crate::win::{enum_processes,check_process,scan_process,filter_process,update_process,write_process};
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

    pub mismem_input: Input,
    pub selected_address: String,
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

            mismem_input: Input::default(),
            selected_address: String::new(),
        };

        app.search_mode.select(Some(0));
        app.search_datatype.select(Some(0));
        app.search_type.select(Some(0));

        app.update();
        app
    }

    pub fn next(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(self.processes.len() - 1) + 1) % self.processes.len()
        ));

    }

    pub fn previous(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.processes.len() + self.table_state.selected().unwrap_or(0) - 1) % self.processes.len()
        ));
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
        macro_rules! popup_error{
            ($e:expr)=>{{
                self.popup_error = format!("Parsing error: {}", $e);
                self.show_popup = true; 
                return;
            }}
        }

        macro_rules! parse{
            ($t:ty,$d:expr)=>{ 
                match self.search_input.value().parse::<$t>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d),
                    Err(e) => popup_error!(e)
                }
            };
            ($t1:ty,$d1:expr;$t2:ty,$d2:expr)=>{ 
                match self.search_input.value().parse::<$t1>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d1),
                    Err(_) => match self.search_input.value().parse::<$t2>() {
                        Ok(r) => (r.to_ne_bytes().to_vec(), $d2),
                        Err(e) => popup_error!(e)
                    }
                }
            }
        }

        // DATATYPE_OPTS = ["Byte", "2 Bytes","4 Bytes","8 Bytes","16 Bytes","Float","Double"];
        let (value_bytes, datatype) = match self.search_datatype.selected().unwrap_or(0) {
            0 => parse!(u8, Datatype::B1; i8, Datatype::B1S), // Byte
            1 => parse!(u16, Datatype::B2; i16, Datatype::B2S), // 2 Bytes
            2 => parse!(u32, Datatype::B4; i32, Datatype::B4S), // 4 Bytes,
            3 => parse!(u64, Datatype::B8; i64, Datatype::B8S), // 8 Bytes
            4 => parse!(u128, Datatype::B16; i128, Datatype::B16S), // 16 Bytes
            5 => parse!(f32, Datatype::F), // Float
            6 => parse!(f64, Datatype::D), // Double
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

    
    pub fn update_memory(&mut self) {
        update_process(self.selected_process, &mut self.memory, &mut self.search_progress);
    }

    pub fn mismem(&mut self) {
        macro_rules! popup_error{
            ($e:expr)=>{{
                self.popup_error = format!("Parsing error: {}", $e);
                self.show_popup = true; 
                return;
            }}
        }

        macro_rules! parse{
            ($t:ty,$d:expr)=>{ 
                match self.mismem_input.value().parse::<$t>() {
                    Ok(r) => (r.to_ne_bytes().to_vec(), $d),
                    Err(e) => popup_error!(e)
                }
            };
        }

        let mut tokens = self.selected_address.split(':');
        let address = usize::from_str_radix(tokens.next().unwrap(), 16).unwrap();
        let (new_value_bytes, datatype)  = match tokens.next().unwrap() {
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
        
        self.update_memory();
        self.edit_state = EditState::Select;

        if !write_process(self.selected_process, address, &new_value_bytes, &datatype) {
            self.popup_error = String::from("Error: can't write at target address.");
            self.show_popup = true; 
        }
    }

    // EditMem

    pub fn memory_next(&mut self) {
        if self.show_popup { return; }

        self.table_state.select(Some(
            (self.table_state.selected().unwrap_or(self.memory.len() - 1) + 1) % self.memory.len()
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