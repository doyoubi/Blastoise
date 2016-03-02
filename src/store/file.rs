use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::vec::Vec;
use std::mem::size_of;
use std::ptr::{write, read, write_bytes};
use std::fs::{OpenOptions, File};
use std::slice::from_raw_parts;
use std::io::{Read, Write, Seek, SeekFrom};
use ::utils::libwrapper::get_page_size;
use ::utils::pointer::{read_string, write_string, pointer_offset};
use ::parser::common::{ValueList, ValueType};
use super::buffer::{DataPtr, PageRef, PagePool};
use super::table::{TableRef, AttrType};
use super::tuple::TupleDesc;


#[derive(Debug)]
pub struct PageHeader {
    pub slot_sum : usize,
    pub first_free_slot : usize,
    pub data : DataPtr,
}

impl PageHeader {
    pub fn save_to_page_data(&mut self) {
        unsafe{
            write::<u32>(self.data as *mut u32, self.slot_sum as u32);
            let next_data_ptr = pointer_offset(self.data, size_of::<u32>());
            write::<u32>(next_data_ptr as *mut u32, self.first_free_slot as u32);
        }
    }
    pub fn init_from_page_data(&mut self) {
        unsafe{
            let slot_sum = read::<u32>(self.data as *const u32) as usize;
            assert_eq!(slot_sum, self.slot_sum);
            let next_data_ptr = pointer_offset(self.data, size_of::<u32>());
            self.first_free_slot = read::<u32>(next_data_ptr as *const u32) as usize;
        }
    }
}

#[derive(Debug)]
pub struct BitMap {
    pub data : DataPtr,
    pub slot_sum : usize,
}

impl BitMap {
    pub fn get_first_free_slot(&self) -> usize {
        let mut count = 0;
        while count < (self.slot_sum + 7) / 8 {
            let n = unsafe{ read::<u8>((self.data as *const u8).offset(count as isize)) };
            if n == 255 { count += 1; continue; }
            let mut mask = 1;
            let mut bit_count = 0;
            loop {
                if (255 - mask) | n < 255 {
                    return count * 8 + bit_count;
                }
                mask *= 2;
                bit_count += 1;
            }
        }
        self.slot_sum
    }
    pub fn get_byte_size(&self) -> usize {
        (self.slot_sum + 7) / 8
    }
    pub fn clean(&mut self) {
        unsafe{
            write_bytes(self.data, 0, self.get_byte_size());
        }
    }
    pub fn set_inuse(&mut self, index : usize, inuse : bool) {
        assert!(index < self.slot_sum);
        let byte_offset = index / 8;
        let bit_offset = index % 8;
        let n = 1 << bit_offset;
        unsafe{
            let p = pointer_offset(self.data, byte_offset);
            let a = read::<u8>(p as *const u8);
            let b = if inuse {
                a | n
            } else {
                a & (255 - n)
            };
            write::<u8>(p as *mut u8, b);
        }
    }
    pub fn is_inuse(&self, index : usize) -> bool {
        assert!(index < self.slot_sum);
        let byte_offset = index / 8;
        let bit_offset = index % 8;
        let n = 1 << bit_offset;
        unsafe{
            let p = pointer_offset(self.data, byte_offset);
            let a = read::<u8>(p as *const u8);
            (a & n) > 0
        }
    }
}

#[derive(Debug)]
pub struct FilePage {
    pub header : PageHeader,
    pub bitmap : BitMap,
    pub tuple_data : DataPtr,
    pub mem_page : PageRef,
}

impl FilePage {
    pub fn new(mem_page : PageRef, tuple_len : usize) -> FilePage {
        let data = mem_page.write().unwrap().data;
        let header_size = 2 * size_of::<u32>();  // PageHeader
        let page_size = get_page_size();
        // (n + 8 - 1) / 8 + tuple_len * n <= page_size - header_size
        let slot_sum = (8 * (page_size - header_size) - 7) / (8 * tuple_len + 1);
        let bitmap_data = unsafe{ data.offset(header_size as isize) };
        let bitmap_size = (slot_sum + 7) / 8;
        let tuple_data = unsafe{ bitmap_data.offset(bitmap_size as isize) };
        FilePage{
            header : PageHeader{
                slot_sum : slot_sum,
                first_free_slot : 0,
                data : data,
            },
            bitmap : BitMap{
                data : bitmap_data,
                slot_sum : slot_sum,
            },
            tuple_data : tuple_data,
            mem_page : mem_page,
        }
    }
    pub fn init_empty_page(&mut self) {
        let _lock = self.mem_page.write().unwrap();
        self.header.save_to_page_data();
        self.bitmap.clean();
    }
    pub fn init_from_page_data(&mut self) {
        let _lock = self.mem_page.read().unwrap();
        self.header.init_from_page_data();
    }
    pub fn save_to_page(&mut self) {
        let _lock = self.mem_page.write().unwrap();
        self.header.save_to_page_data();
    }
    pub fn is_inuse(&self, index : usize) -> bool {
        let _lock = self.mem_page.write().unwrap();
        self.bitmap.is_inuse(index)
    }
    pub fn set_inuse(&mut self, index : usize, inuse : bool) {
        let _lock = self.mem_page.write().unwrap();
        self.bitmap.set_inuse(index, inuse);
    }
    pub fn insert(&mut self, value_list : &ValueList, tuple_desc : &TupleDesc) {
        assert!(!self.is_inuse(self.header.first_free_slot));
        assert_eq!(value_list.len(), tuple_desc.attr_desc.len());
        assert!(self.header.first_free_slot < self.bitmap.slot_sum);

        let first_free_slot = self.header.first_free_slot;
        self.set_inuse(first_free_slot, true);
        self.header.first_free_slot = self.bitmap.get_first_free_slot();
        self.save_to_page();

        let _lock = self.mem_page.read().unwrap();
        let mut p = unsafe{
            self.tuple_data.offset(
                (tuple_desc.tuple_len * first_free_slot) as isize
                )
        };
        for (v, d) in value_list.iter().zip(&tuple_desc.attr_desc) {
            match (v.value_type, d) {
                (ValueType::Integer, &AttrType::Int) => {
                    let n : i32 = v.value.parse::<i32>().unwrap();
                    unsafe{ write::<i32>(p as *mut i32, n) };
                    p = pointer_offset(p, 4);
                }
                (ValueType::Float, &AttrType::Float) | (ValueType::Integer, &AttrType::Float) => {
                    let n : f32 = v.value.parse::<f32>().unwrap();
                    unsafe{ write::<f32>(p as *mut f32, n) };
                    p = pointer_offset(p, 4);
                }
                (ValueType::String, &AttrType::Char{len}) => {
                    let aligned_len = (len + 3) / 4 * 4;
                    write_string(p, &v.value, len);
                    p = pointer_offset(p, aligned_len);
                }
                (ValueType::Null, &AttrType::Int) | (ValueType::Null, &AttrType::Float) => {
                    unsafe{ write_bytes(p, 0, 4) };
                    p = pointer_offset(p, 4);
                }
                (ValueType::Null, &AttrType::Char{len}) => {
                    let aligned_len = (len + 3) / 4 * 4;
                    unsafe{ write_bytes(p, 0, aligned_len) };
                    p = pointer_offset(p, aligned_len);
                }
                _ => panic!("invalid value, expected {:?}, found {:?}", d, v),
            }
        }
    }
    pub fn is_full(&self) -> bool {
        self.header.first_free_slot == self.bitmap.slot_sum
    }
}


type TableFileRef = Arc<RwLock<TableFile>>;

#[derive(Debug)]
pub struct TableFile {
    pub saved_name : String,
    pub file : File,
    pub page_list : Vec<FilePage>,
    pub table : TableRef,
    pub first_free_page : usize,
    pub tuple_desc : TupleDesc,  // for FilePage
}

impl TableFile {
    pub fn new(name : &String, table : TableRef) -> TableFile {
        let file = OpenOptions::new().read(true).write(true).create(true).open(name).unwrap();
        let tuple_desc = table.read().unwrap().gen_tuple_desc();
        TableFile{
            saved_name : name.clone(),
            file : file,
            page_list : Vec::new(),
            table : table,
            first_free_page : 0,
            tuple_desc : tuple_desc,
        }
    }
    pub fn save_to_file(&mut self) {
        is_match!(self.file.seek(SeekFrom::Start(0)), Ok(..));
        let header = [self.page_list.len() as u32, self.first_free_page as u32];
        is_match!(self.file.write_all(unsafe{
            from_raw_parts::<u8>((&header).as_ptr() as *const u8, 8)
        }), Ok(..));
        // TODO: write page data to file
    }
    pub fn init_from_file(&mut self) {}
    pub fn insert(&mut self, value_list : &ValueList) {
        // must call add_page first if need_new_page() is true
        assert!(self.first_free_page < self.page_list.len());
        let file_page = &mut self.page_list[self.first_free_page];
        file_page.insert(value_list, &self.tuple_desc)
    }
    pub fn need_new_page(&mut self) -> bool {
        while self.first_free_page < self.page_list.len() {
            let page = &self.page_list[self.first_free_page];
            if page.is_full() {
                self.first_free_page += 1
            } else {
                break;
            }
        }
        self.first_free_page == self.page_list.len()
    }
    pub fn add_page(&mut self, mem_page : PageRef) {
        let file_page = FilePage::new(mem_page, self.tuple_desc.tuple_len);
        self.page_list.push(file_page);
    }
}


#[derive(Debug)]
pub struct TableFileManager {
    files : HashMap<String, TableFileRef>,  // key is table name
    page_pool : PagePool,
}

impl TableFileManager {
    fn new(pool_capacity : usize) -> TableFileManager {
        TableFileManager{
            files : HashMap::new(),
            page_pool : PagePool::new(pool_capacity),
        }
    }
    fn from_files(_path : &str) -> TableFileManager {
        unimplemented!()
    }
    fn get_file(&self, table : &String) -> TableFileRef {
        self.files.get(table).unwrap().clone()
    }
    fn create_file(&mut self, name : &String, table : TableRef) {
        let file = Arc::new(RwLock::new(TableFile::new(name, table)));
        self.files.insert(name.clone(), file);
    }
}
