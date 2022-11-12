use std::{
    mem::{size_of_val, size_of}, 
    ops::BitAnd, 
    convert::TryInto, 
    sync::Arc
};

use tokio::sync::Mutex;

use windows::Win32::{
    Foundation::{
        HINSTANCE, HANDLE,
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
            WriteProcessMemory,
        },
    },
};

use crate::app::App;
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
        K32EnumProcesses(pids.as_mut_ptr(), size_of_val(&pids) as u32, &mut np);
        for pid in &pids[..np as usize] {
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, *pid);

            if process.is_ok() {
                let process = process.unwrap();
                let mut module = HINSTANCE::default();
                let mut cb = 0;
                
                if K32EnumProcessModules(process, &mut module, size_of_val(&module) as u32, &mut cb).as_bool()
                {
                    // Get Process Name
                    let mut name: [u16; 512] = [0; 512];
                    let len = K32GetModuleBaseNameW(process, module, &mut name);
                    let name = String::from_utf16_lossy(&name[..len as usize]);
                    
                    // Get Process Memory Usage
                    let mut pmemcounters = PROCESS_MEMORY_COUNTERS::default();
                    let mem_usage = if K32GetProcessMemoryInfo(process, &mut pmemcounters, size_of_val(&pmemcounters) as u32).as_bool() {
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


struct MemInfo<T> {
    base: *const T,
    size: usize,
}
unsafe impl<T> Send for MemInfo<T> {}
unsafe impl<T> Sync for MemInfo<T> {}


pub async fn scan_process(pid : u32, target_bytes: &[u8], target_type: &Datatype, app_mutex: Arc<Mutex<App>>) {
    let mut results = Memory::new();
    let num_bytes = target_bytes.len();

    match unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid) } 
    {
        Ok(process) => {
            let mut app = app_mutex.lock().await;
            app.memory = Memory::new();
            drop(app);

            let pages = get_process_memory_pages(process).iter()
                .map(|&x| MemInfo{base: x.BaseAddress as *const _, size: x.RegionSize})
                .collect::<Vec<_>>();
            
            let mut sweeped_memory : usize = 0;
            let total_memory = pages.iter().map(|p| p.size).sum::<usize>() as f64;

            for page in pages.iter() {
                let mut buffer: Vec<u8> = Vec::with_capacity(page.size);
                let mut bytes_read: usize = 0;

                unsafe { ReadProcessMemory(
                    process,
                    page.base,
                    buffer.as_mut_ptr() as *mut _,
                    page.size,
                    Some(&mut bytes_read)
                ) };

                if page.size == bytes_read 
                {
                    unsafe { buffer.set_len(bytes_read) };
    
                    buffer.windows(num_bytes).enumerate().for_each(|(offset, window)| {
                        if window == target_bytes {
                            results.push(page.base as usize + offset, target_type, target_bytes);
                        }
                    });
                }

                sweeped_memory += page.size;
                let mut app = app_mutex.lock().await;
                app.search_progress = sweeped_memory as f64 / total_memory;
            }

            unsafe { CloseHandle(process) };

            let mut app = app_mutex.lock().await;
            app.search_progress = 1f64;
            app.memory = std::mem::take(&mut results);
            log::info!(" First Scan found {} entries.", app.memory.len());
        },
        Err(error) => {
            log::error!("Error while analyzing process: {:?}", error);
        }
    }
}


pub async fn filter_process(pid : u32, target_bytes: &[u8], target_type: &Datatype, app_mutex: Arc<Mutex<App>>) {
    let num_bytes = target_bytes.len();

    match unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid) } 
    {
        Ok(process) => {
            let mut app = app_mutex.lock().await;
            let mut memory = std::mem::take(&mut app.memory);
            drop(app);

            let mut buffer: Vec<u8> = vec![0;num_bytes];
            let mut bytes_read: usize = 0;
            
            let mut sweeped_memory : usize = 0;
            let total_memory = memory.len();
            let progress_update_freq = std::cmp::max(total_memory, total_memory / 100);

            macro_rules! filter_mem_type{
                ($($a:ident).+,$b:ty)=>{
                    {                        
                        $($a).+.retain_mut(|l| {
                            unsafe { ReadProcessMemory(process, l.address as *const _, buffer.as_mut_ptr() as *mut _, target_bytes.len(), Some(&mut bytes_read)) };
                            l.old_value = l.value;
                            l.value = <$b>::from_ne_bytes(buffer.clone().try_into().unwrap());
                            sweeped_memory += 1;

                            if sweeped_memory % progress_update_freq == 0 {
                                if let Ok(mut app) = app_mutex.try_lock() {
                                    app.search_progress = sweeped_memory as f64 / total_memory as f64; 
                                }
                            }
                            bytes_read == num_bytes && target_bytes == buffer
                        });
                    }
                }
            }

            match *target_type {
                Datatype::B1 => filter_mem_type![memory.mem_u8,u8],
                Datatype::B1S => filter_mem_type![memory.mem_i8,i8],
                Datatype::B2 => filter_mem_type![memory.mem_u16,u16],
                Datatype::B2S => filter_mem_type![memory.mem_i16,i16],
                Datatype::B4 => filter_mem_type![memory.mem_u32,u32],
                Datatype::B4S => filter_mem_type![memory.mem_i32,i32],
                Datatype::B8 => filter_mem_type![memory.mem_u64,u64],
                Datatype::B8S => filter_mem_type![memory.mem_i64,i64],
                Datatype::B16 => filter_mem_type![memory.mem_u128,u128],
                Datatype::B16S => filter_mem_type![memory.mem_i128,i128],
                Datatype::F => filter_mem_type![memory.mem_f32,f32],
                Datatype::D => filter_mem_type![memory.mem_f64,f64],
            }

            unsafe { CloseHandle(process) };

            let mut app = app_mutex.lock().await;
            app.memory = std::mem::take(&mut memory);
            app.search_progress = 1f64;
            log::info!(" {} entries remaining after filtering.", app.memory.len());
        },
        Err(error) => {
            log::error!("Error while analyzing process: {:?}", error);
        }
    }
}


pub async fn update_process(app_mutex : Arc<Mutex<App>>) {
    let mut app = app_mutex.lock().await;
    let pid = app.selected_process;

    match unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid) }
    {
        Ok(process) => {
            let mut memory = std::mem::take(&mut app.memory);
            drop(app);

            let mut i : usize = 0;
            let memory_size = memory.len();
            let progress_update_freq = std::cmp::max(memory_size, memory_size / 100);

            macro_rules! update_mem_type{
                ($($a:ident).+,$b:ty)=>{
                    {
                        let num_bytes = <$b>::default().to_ne_bytes().len();
                        let mut buffer: Vec<u8> = vec![0;num_bytes];
                        let mut bytes_read: usize = 0;

                        $($a).+.retain_mut(|l| {
                            unsafe { ReadProcessMemory(process, l.address as *const _, buffer.as_mut_ptr() as *mut _, num_bytes, Some(&mut bytes_read)) };
                            l.old_value = l.value;
                            l.value = <$b>::from_ne_bytes(buffer.clone().try_into().unwrap());
                            i += 1;
                            
                            if i % progress_update_freq == 0 {
                                if let Ok(mut app) = app_mutex.try_lock() {
                                    app.search_progress = i as f64 / memory_size as f64; 
                                }
                            }
                            bytes_read == num_bytes
                        });
                    }
                }
            }

            update_mem_type![memory.mem_u8,u8];
            update_mem_type![memory.mem_i8,i8];
            update_mem_type![memory.mem_u16,u16];
            update_mem_type![memory.mem_i16,i16];
            update_mem_type![memory.mem_u32,u32];
            update_mem_type![memory.mem_i32,i32];
            update_mem_type![memory.mem_u64,u64];
            update_mem_type![memory.mem_i64,i64];
            update_mem_type![memory.mem_u128,u128];
            update_mem_type![memory.mem_i128,i128];
            update_mem_type![memory.mem_f32,f32];
            update_mem_type![memory.mem_f64,f64];

            unsafe { CloseHandle(process) };

            let mut app = app_mutex.lock().await;
            app.search_progress = 1f64;
            app.memory = std::mem::take(&mut memory);
        },
        Err(error) => {
            log::error!("Error while analyzing process: {:?}", error);
        }
    }
}


pub fn write_process(pid : u32, address : usize, target_bytes: &[u8]) -> bool {
    let num_bytes = target_bytes.len();

    unsafe { 
        match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid)
        {
            Ok(process) => {
                let mut bytes_written: usize = 0;

                WriteProcessMemory(
                    process,
                    address as *const _,
                    target_bytes.to_vec().as_ptr() as *const _,
                    num_bytes,
                    Some(&mut bytes_written)
                ); 

                CloseHandle(process);

                bytes_written == num_bytes
            },
            Err(_) => {
                false
            }
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
