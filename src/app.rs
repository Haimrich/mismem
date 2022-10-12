use std::mem;

use tui::widgets::TableState;

use windows::Win32::{
    System::ProcessStatus::{
        K32EnumProcessModules,
        K32GetModuleBaseNameA,
    },
    Foundation::{
        HINSTANCE, 
        CloseHandle,
    }, 
    System::Threading::{
        OpenProcess,
        PROCESS_QUERY_INFORMATION,
        PROCESS_VM_READ
    },
};


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
        App {
            state: AppState::Home,
            table_state: TableState::default(),
            items: vec![
                vec![String::from("Row11"), String::from("Row12"), String::from("Row13")],
                vec![String::from("Row61"), String::from("Row62\nTest"), String::from("Row63")],
            ],
        }
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

    fn get_process_name(pid: u32) -> String {
        let mut process_name = String::from("N/D");
        unsafe {
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);

            if process.is_ok() {
                let process = process.unwrap();
                let mut module = HINSTANCE::default();
                let mut cb = 0;
                
                let module_ok = K32EnumProcessModules(process, &mut module, mem::size_of_val(&module) as u32, &mut cb).as_bool();
                
                if module_ok
                {
                    let mut name_bytes = [0; 1024];
                    K32GetModuleBaseNameA(process, module, name_bytes.as_mut_slice());
                    process_name = String::from_utf8(name_bytes.to_vec()).unwrap()
                }
                CloseHandle(process);
            }
        }

        process_name
    }

    pub fn update(&mut self) {
        self.items.clear();

        let mut process_vec: [u32; 1024] = [0; 1024];
        let mut np: u32 = 0;
        unsafe {
            windows::Win32::System::ProcessStatus::K32EnumProcesses(process_vec.as_mut_ptr(), mem::size_of_val(&process_vec) as u32, &mut np);
            for p in process_vec {
                if p != 0 {
                    // let name = windows::Win32::System::ProcessStatus::PrintProcessNameAnd
                    let v = p.to_string();
                    self.items.push(vec![Self::get_process_name(p), v.clone(), v.clone()]);
                }
            }
        }
    }
}