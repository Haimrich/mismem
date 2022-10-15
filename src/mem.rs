use std::{fmt, convert::TryInto};

// TODO compact

pub enum Datatype {
    B16,
    B16S,
    B8,
    B8S,
    B4,
    B4S,
    B2,
    B2S,
    B1,
    B1S,
    D,
    F,
}

pub struct Location<T: fmt::Display> {
    pub address: usize,
    pub value: T,
    pub old_value: T,
}

pub struct Memory {

    pub mem_i128: Vec<Location<i128>>, 
    pub mem_u128: Vec<Location<u128>>, 

    pub mem_i64: Vec<Location<i64>>, 
    pub mem_u64: Vec<Location<u64>>, 

    pub mem_i32: Vec<Location<i32>>, 
    pub mem_u32: Vec<Location<u32>>,

    pub mem_i16: Vec<Location<i16>>, 
    pub mem_u16: Vec<Location<u16>>, 

    pub mem_i8: Vec<Location<i8>>, 
    pub mem_u8: Vec<Location<u8>>, 

    pub mem_f64: Vec<Location<f64>>,
    pub mem_f32: Vec<Location<f32>>, 

}

impl Memory {
    pub fn new() -> Memory {
        Memory { 
            mem_i128: Vec::<Location<i128>>::new(), 
            mem_u128: Vec::<Location<u128>>::new(), 
        
            mem_i64: Vec::<Location<i64>>::new(), 
            mem_u64: Vec::<Location<u64>>::new(), 
        
            mem_i32: Vec::<Location<i32>>::new(), 
            mem_u32: Vec::<Location<u32>>::new(),
        
            mem_i16: Vec::<Location<i16>>::new(), 
            mem_u16: Vec::<Location<u16>>::new(), 
        
            mem_i8: Vec::<Location<i8>>::new(), 
            mem_u8: Vec::<Location<u8>>::new(), 
        
            mem_f64: Vec::<Location<f64>>::new(),
            mem_f32: Vec::<Location<f32>>::new(), 
         }
    }

    pub fn clear(&mut self) {
        self.mem_i128.clear();
        self.mem_u128.clear();
    
        self.mem_i64.clear();
        self.mem_u64.clear(); 
    
        self.mem_i32.clear();
        self.mem_u32.clear();
    
        self.mem_i16.clear();
        self.mem_u16.clear();
    
        self.mem_i8.clear(); 
        self.mem_u8.clear();
    
        self.mem_f64.clear();
        self.mem_f32.clear();
    }

    pub fn len(&mut self) -> usize {
        self.mem_i128.len() + self.mem_u128.len() + self.mem_i64.len() + self.mem_u64.len() + self.mem_i32.len() + self.mem_u32.len() +
        self.mem_i16.len() + self.mem_u16.len() + self.mem_i8.len() + self.mem_u8.len() + self.mem_f64.len() + self.mem_f32.len()
    }

    pub fn push(&mut self, address: usize, target_type: &Datatype, target_bytes: &[u8]) {
        match *target_type {
            Datatype::B1 => {
                let value = u8::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_u8.push(Location::<u8>{address: address, value: value, old_value: value});
            },
            Datatype::B1S => {
                let value = i8::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_i8.push(Location::<i8>{address: address, value: value, old_value: value});
            },
            Datatype::B2 => {
                let value = u16::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_u16.push(Location::<u16>{address: address, value: value, old_value: value});
            },
            Datatype::B2S => {
                let value = i16::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_i16.push(Location::<i16>{address: address, value: value, old_value: value});
            },
            Datatype::B4 => {
                let value = u32::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_u32.push(Location::<u32>{address: address, value: value, old_value: value});
            },
            Datatype::B4S => {
                let value = i32::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_i32.push(Location::<i32>{address: address, value: value, old_value: value});
            },
            Datatype::B8 => {
                let value = u64::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_u64.push(Location::<u64>{address: address, value: value, old_value: value});
            },
            Datatype::B8S => {
                let value = i64::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_i64.push(Location::<i64>{address: address, value: value, old_value: value});
            },
            Datatype::B16 => {
                let value = u128::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_u128.push(Location::<u128>{address: address, value: value, old_value: value});
            },
            Datatype::B16S => {
                let value = i128::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_i128.push(Location::<i128>{address: address, value: value, old_value: value});
            },
            Datatype::F => {
                let value = f32::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_f32.push(Location::<f32>{address: address, value: value, old_value: value});
            },
            Datatype::D => {
                let value = f64::from_ne_bytes(target_bytes.try_into().unwrap());
                self.mem_f64.push(Location::<f64>{address: address, value: value, old_value: value});
            }
        }
    }

    pub fn iter(&self) -> MemoryIterator {
        MemoryIterator { memory: self, curs: [0;12] }
    }
/* 
    pub fn get_type_mem_bytes(&self, datatype : &Datatype) -> impl Iterator<Item = usize> {
        match *datatype {
            Datatype::B1 => self.mem_u8.iter().map(|d| d.address)
        }
    }
    */
}

pub struct MemoryIterator<'a> {
    memory : &'a Memory,
    curs : [usize;12],
}

impl<'a> Iterator for MemoryIterator<'a> {
    type Item = [String;3];

    fn next(&mut self) -> Option<Self::Item> {
        let mut min_address = usize::MAX;
        let mut min_type_id = 0;

        if self.curs[0] < self.memory.mem_i128.len() {
            if self.memory.mem_i128[self.curs[0]].address < min_address {
                min_address = self.memory.mem_i128[self.curs[0]].address;
                min_type_id = 0;
            }
        }
        if self.curs[1] < self.memory.mem_u128.len() {
            if self.memory.mem_u128[self.curs[1]].address < min_address {
                min_address = self.memory.mem_u128[self.curs[1]].address;
                min_type_id = 1;
            }
        }
        if self.curs[2] < self.memory.mem_i64.len() {
            if self.memory.mem_i64[self.curs[2]].address < min_address {
                min_address = self.memory.mem_i64[self.curs[2]].address;
                min_type_id = 2;
            }
        }
        if self.curs[3] < self.memory.mem_u64.len() {
            if self.memory.mem_u64[self.curs[3]].address < min_address {
                min_address = self.memory.mem_u64[self.curs[3]].address;
                min_type_id = 3;
            }
        }
        if self.curs[4] < self.memory.mem_i32.len() {
            if self.memory.mem_i32[self.curs[4]].address < min_address {
                min_address = self.memory.mem_i32[self.curs[4]].address;
                min_type_id = 4;
            }
        }
        if self.curs[5] < self.memory.mem_u32.len() {
            if self.memory.mem_u32[self.curs[5]].address < min_address {
                min_address = self.memory.mem_u32[self.curs[5]].address;
                min_type_id = 5;
            }
        }
        if self.curs[6] < self.memory.mem_i16.len() {
            if self.memory.mem_i16[self.curs[6]].address < min_address {
                min_address = self.memory.mem_i16[self.curs[6]].address;
                min_type_id = 6;
            }
        }
        if self.curs[7] < self.memory.mem_u16.len() {
            if self.memory.mem_u16[self.curs[7]].address < min_address {
                min_address = self.memory.mem_u16[self.curs[7]].address;
                min_type_id = 7;
            }
        }
        if self.curs[8] < self.memory.mem_i8.len() {
            if self.memory.mem_i8[self.curs[8]].address < min_address {
                min_address = self.memory.mem_i8[self.curs[8]].address;
                min_type_id = 8;
            }
        }
        if self.curs[9] < self.memory.mem_u8.len() {
            if self.memory.mem_u8[self.curs[9]].address < min_address {
                min_address = self.memory.mem_u8[self.curs[9]].address;
                min_type_id = 9;
            }
        }
        if self.curs[10] < self.memory.mem_f64.len() {
            if self.memory.mem_f64[self.curs[10]].address < min_address {
                min_address = self.memory.mem_f64[self.curs[10]].address;
                min_type_id = 10;
            }
        }
        if self.curs[11] < self.memory.mem_f32.len() {
            if self.memory.mem_f32[self.curs[11]].address < min_address {
                min_address = self.memory.mem_f32[self.curs[11]].address;
                min_type_id = 11;
            }
        }

        let next = if min_address != usize::MAX {
            match min_type_id {
                0 => Some([
                    format!("{:#016X}", self.memory.mem_i128[self.curs[min_type_id]].address),
                    self.memory.mem_i128[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i128[self.curs[min_type_id]].old_value.to_string(),
                ]),
                1 => Some([
                    format!("{:#016X}", self.memory.mem_u128[self.curs[min_type_id]].address),
                    self.memory.mem_u128[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_u128[self.curs[min_type_id]].old_value.to_string(),
                ]),
                2 => Some([
                    format!("{:#016X}", self.memory.mem_i64[self.curs[min_type_id]].address),
                    self.memory.mem_i64[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i64[self.curs[min_type_id]].old_value.to_string(),
                ]),
                3 => Some([
                    format!("{:#016X}", self.memory.mem_u64[self.curs[min_type_id]].address),
                    self.memory.mem_u64[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_u64[self.curs[min_type_id]].old_value.to_string(),
                ]),
                4 => Some([
                    format!("{:#016X}", self.memory.mem_i32[self.curs[min_type_id]].address),
                    self.memory.mem_i32[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i32[self.curs[min_type_id]].old_value.to_string(),
                ]),
                5 => Some([
                    format!("{:#016X}", self.memory.mem_u32[self.curs[min_type_id]].address),
                    self.memory.mem_i32[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i32[self.curs[min_type_id]].old_value.to_string(),
                ]),
                6 => Some([
                    format!("{:#016X}", self.memory.mem_i16[self.curs[min_type_id]].address),
                    self.memory.mem_i16[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i16[self.curs[min_type_id]].old_value.to_string(),
                ]),
                7 => Some([
                    format!("{:#016X}", self.memory.mem_u16[self.curs[min_type_id]].address),
                    self.memory.mem_u16[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_u16[self.curs[min_type_id]].old_value.to_string(),
                ]),
                8 => Some([
                    format!("{:#016X}", self.memory.mem_i8[self.curs[min_type_id]].address),
                    self.memory.mem_i8[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_i8[self.curs[min_type_id]].old_value.to_string(),
                ]),
                9 => Some([
                    format!("{:#016X}", self.memory.mem_u8[self.curs[min_type_id]].address),
                    self.memory.mem_u8[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_u8[self.curs[min_type_id]].old_value.to_string(),
                ]),
                10 => Some([
                    format!("{:#016X}", self.memory.mem_f64[self.curs[min_type_id]].address),
                    self.memory.mem_f64[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_f64[self.curs[min_type_id]].old_value.to_string(),
                ]),
                11 => Some([
                    format!("{:#016X}", self.memory.mem_f32[self.curs[min_type_id]].address),
                    self.memory.mem_f32[self.curs[min_type_id]].value.to_string(),
                    self.memory.mem_f32[self.curs[min_type_id]].old_value.to_string(),
                ]),
                _ => None
            }
        } else {
            None
        };

        self.curs[min_type_id] += 1;

        next
    }
}
