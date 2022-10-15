use std::{mem::{self, size_of}, ops::BitAnd, convert::TryInto};

use windows::Win32::{
    Foundation::{
        BOOL, HWND, LPARAM, HINSTANCE, HANDLE,
        CloseHandle,
    }, 
    System::{
        ProcessStatus::{
            K32EnumProcesses,
            K32EnumProcessModules,
            K32GetModuleBaseNameW,
            K32GetProcessMemoryInfo,
            PROCESS_MEMORY_COUNTERS,
        },
        Threading::{
            OpenProcess,
            PROCESS_QUERY_INFORMATION,
            PROCESS_VM_READ,
            PROCESS_VM_WRITE,
        },
        Memory::{
            VirtualQueryEx,
            MEMORY_BASIC_INFORMATION,
            PAGE_READWRITE,
            MEM_COMMIT,
        },
        Diagnostics::Debug::{
            ReadProcessMemory,
        },
    },
    UI::WindowsAndMessaging::{
        EnumWindows, 
        GetWindowTextW
    },
};

use crate::mem::{Memory,Datatype};


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


pub fn check_process(pid : u32) -> bool {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid);

        if process.is_ok() {
            CloseHandle(process.unwrap());
            true
        } else {
            false
        }
    }
}


pub fn scan_process(pid : u32, target_bytes: &[u8], target_type: &Datatype, progress: &mut f64) -> Memory {
    let mut results = Memory::new();
    let num_bytes = target_bytes.len();

    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid);

        if process.is_ok() {
            let hprocess = process.unwrap();
            let pages = get_process_memory_pages(hprocess);

            for (i, page) in pages.iter().enumerate() {

                let mut buffer: Vec<u8> = Vec::with_capacity(page.RegionSize);
                let mut bytes_read: usize = 0;

                ReadProcessMemory(
                    hprocess,
                    page.BaseAddress as *const _,
                    buffer.as_mut_ptr() as *mut _,
                    page.RegionSize,
                    Some(&mut bytes_read)
                );

                if page.RegionSize != bytes_read {
                    continue;
                }
                buffer.set_len(bytes_read);

                buffer.windows(num_bytes).enumerate().for_each(|(offset, window)| {
                    if window == target_bytes {
                        results.push(page.BaseAddress as usize + offset, target_type, target_bytes);
                    }
                });

                *progress = (i+1) as f64 / pages.len() as f64;
            }


            CloseHandle(hprocess);
        }
    }
    results
}



pub fn filter_process(pid : u32, memory : &mut Memory, target_bytes: &[u8], target_type: &Datatype, progress: &mut f64) {
    let num_bytes = target_bytes.len();

    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid);

        if process.is_ok() {
            let hprocess = process.unwrap();

            let mut buffer: Vec<u8> = vec![0;num_bytes];
            let mut bytes_read: usize = 0;
            let mut i = 0f64;

            macro_rules! update_mem_type{
                ($($a:ident).+,$b:ty)=>{
                    {
                        let total = $($a).+.len() as f64;
                        $($a).+.retain_mut(|l| {
                            ReadProcessMemory(hprocess, l.address as *const _, buffer.as_mut_ptr() as *mut _, target_bytes.len(), Some(&mut bytes_read));
                            l.old_value = l.value;
                            l.value = <$b>::from_ne_bytes(buffer.clone().try_into().unwrap());
                            i = i + 1.0;
                            *progress = i / total;
                            bytes_read == num_bytes && target_bytes == buffer
                        });
                    }
                }
            }

            match *target_type {
                Datatype::B1 => update_mem_type![memory.mem_u8,u8],
                Datatype::B1S => update_mem_type![memory.mem_i8,i8],
                Datatype::B2 => update_mem_type![memory.mem_u16,u16],
                Datatype::B2S => update_mem_type![memory.mem_i16,i16],
                Datatype::B4 => update_mem_type![memory.mem_u32,u32],
                Datatype::B4S => update_mem_type![memory.mem_i32,i32],
                Datatype::B8 => update_mem_type![memory.mem_u64,u64],
                Datatype::B8S => update_mem_type![memory.mem_i64,i64],
                Datatype::B16 => update_mem_type![memory.mem_u128,u128],
                Datatype::B16S => update_mem_type![memory.mem_i128,i128],
                Datatype::F => update_mem_type![memory.mem_f32,f32],
                Datatype::D => update_mem_type![memory.mem_f64,f64],
            }

            CloseHandle(hprocess);
        }
    }
}


fn get_process_memory_pages(hprocess : HANDLE) -> Vec<MEMORY_BASIC_INFORMATION> {
    let mut pages = Vec::new();
    let mut lpaddress = 0;
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    const MBI_SIZE : usize = size_of::<MEMORY_BASIC_INFORMATION>();
    unsafe {
        while VirtualQueryEx(hprocess, Some(lpaddress as *const _), &mut mbi, MBI_SIZE) == MBI_SIZE {
            if mbi.AllocationProtect.bitand(PAGE_READWRITE).0 != 0 && mbi.State.bitand(MEM_COMMIT).0 != 0 {
                pages.push(mbi);
            }
            lpaddress += mbi.RegionSize;
        }
    }
    pages
}


/*
pub struct WinApp {
    pub name: String,
    pub memory: f64,
    pub hwnd: HWND,
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
*/
