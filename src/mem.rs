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
        macro_rules! mem_push{
            ($t:ty,$mem:ident)=>{{
                let value = <$t>::from_ne_bytes(target_bytes.try_into().unwrap());
                self.$mem.push(Location::<$t>{address: address, value: value, old_value: value});
            }}
        }

        match *target_type {
            Datatype::B1 => mem_push!(u8,mem_u8),
            Datatype::B1S => mem_push!(i8,mem_i8),
            Datatype::B2 => mem_push!(u16,mem_u16),
            Datatype::B2S => mem_push!(i16,mem_i16),
            Datatype::B4 => mem_push!(u32,mem_u32),
            Datatype::B4S => mem_push!(i32,mem_i32),
            Datatype::B8 => mem_push!(u64,mem_u64),
            Datatype::B8S => mem_push!(i64,mem_i64),
            Datatype::B16 => mem_push!(u128,mem_u128),
            Datatype::B16S => mem_push!(i128,mem_i128),
            Datatype::F => mem_push!(f32,mem_f32),
            Datatype::D => mem_push!(f64,mem_f64)
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

        macro_rules! check_min_address{
            ($mem:ident,$id:expr)=>{ 
                if self.curs[$id] < self.memory.$mem.len() {
                    if self.memory.$mem[self.curs[$id]].address < min_address {
                        min_address = self.memory.$mem[self.curs[$id]].address;
                        min_type_id = $id;
                    }
                }
            }
        }

        check_min_address!(mem_i128,0);
        check_min_address!(mem_u128,1);
        check_min_address!(mem_i64,2);
        check_min_address!(mem_u64,3);
        check_min_address!(mem_i32,4);
        check_min_address!(mem_u32,5);
        check_min_address!(mem_i16,6);
        check_min_address!(mem_u16,7);
        check_min_address!(mem_i8,8);
        check_min_address!(mem_u8,9);
        check_min_address!(mem_f64,10);
        check_min_address!(mem_f32,11);

        macro_rules! get_next_entry{
            ($mem:ident,$suffix:expr)=>{ 
                Some([
                    format!("{:016X}{}", self.memory.$mem[self.curs[min_type_id]].address,$suffix),
                    self.memory.$mem[self.curs[min_type_id]].value.to_string(),
                    self.memory.$mem[self.curs[min_type_id]].old_value.to_string(),
                ])
            };
            ($mem:ident)=>{
                get_next_entry!($mem,"")
            }
        }

        let next = if min_address != usize::MAX {
            match min_type_id {
                0 => get_next_entry!(mem_i128,":i128"),
                1 => get_next_entry!(mem_u128,":u128"),
                2 => get_next_entry!(mem_i64,":i64"),
                3 => get_next_entry!(mem_u64,":u64"),
                4 => get_next_entry!(mem_i32,":i32"),
                5 => get_next_entry!(mem_u32,":u32"),
                6 => get_next_entry!(mem_i16,":i16"),
                7 => get_next_entry!(mem_u16,":u16"),
                8 => get_next_entry!(mem_i8,":i8"),
                9 => get_next_entry!(mem_u8,":u8"),
                10 => get_next_entry!(mem_f64,":f64"),
                11 => get_next_entry!(mem_f32,":f32"),
                _ => None
            }
        } else {
            None
        };

        self.curs[min_type_id] += 1;

        next
    }
}
