use std::mem;

use windows::Win32::{
    Foundation::{
        BOOL, HWND, LPARAM, HINSTANCE, 
        CloseHandle,
    }, 
    System::ProcessStatus::{
        K32EnumProcesses,
        K32EnumProcessModules,
        K32GetModuleBaseNameW,
        K32GetProcessMemoryInfo,
        PROCESS_MEMORY_COUNTERS,
    },
    System::Threading::{
        OpenProcess,
        PROCESS_QUERY_INFORMATION,
        PROCESS_VM_READ
    },
    UI::WindowsAndMessaging::{
        EnumWindows, 
        GetWindowTextW
    },
};

pub struct WinApp {
    pub name: String,
    pub memory: f64,
    pub hwnd: HWND,
}

pub struct WinProc {
    pub name: String,
    pub memory: f64,
    pub pid: u32,
}

pub fn enum_processes() -> Vec<WinProc> {
    let mut processes = Vec::<WinProc>::new();

    let mut pids: [u32; 4096] = [0; 4096];
    let mut np: u32 = 0;
    unsafe {
        K32EnumProcesses(pids.as_mut_ptr(), mem::size_of_val(&pids) as u32, &mut np);
        for pid in &pids[..np as usize] {
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, *pid);

            if process.is_ok() {
                let process = process.unwrap();
                let mut module = HINSTANCE::default();
                let mut cb = 0;
                
                if K32EnumProcessModules(process, &mut module, mem::size_of_val(&module) as u32, &mut cb).as_bool()
                {
                    // Get Process Name
                    let mut name: [u16; 512] = [0; 512];
                    let len = K32GetModuleBaseNameW(process, module, &mut name);
                    let name = String::from_utf16_lossy(&name[..len as usize]);
                    
                    // Get Process Memory Usage
                    let mut pmemcounters = PROCESS_MEMORY_COUNTERS::default();
                    let mem_usage = if K32GetProcessMemoryInfo(process, &mut pmemcounters, mem::size_of_val(&pmemcounters) as u32).as_bool() {
                        (pmemcounters.WorkingSetSize / 1024) as f64
                    } else {
                        0.0
                    };

                    processes.push(WinProc{name: name, memory: mem_usage, pid: *pid});
                }
                CloseHandle(process);
            }
        }
    }
    processes.sort_by(|a, b| b.memory.partial_cmp(&a.memory).unwrap());
    processes
}


pub fn enum_windows() -> Vec<WinApp> {
    let mut wnds = Vec::<WinApp>::new();
    let wnds_ptr = &mut wnds as *mut _;

    unsafe {
        EnumWindows(Some(enum_window), LPARAM(wnds_ptr as isize));
    }

    wnds
}

extern "system" fn enum_window(window: HWND, wnds_ptr : LPARAM) -> BOOL {
    unsafe {
        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(window, &mut text);
        let text = String::from_utf16_lossy(&text[..len as usize]);

        let wnds: &mut Vec<WinApp> = mem::transmute(wnds_ptr);
        
        if !text.is_empty() {
            wnds.push(WinApp{name: text, memory: 0.0, hwnd: window});
        }

        true.into()
    }
}
