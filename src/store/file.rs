use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::vec::Vec;
use std::mem::size_of;
use std::ptr::{write, read, write_bytes};
use std::fs::{OpenOptions, File};
use ::utils::libwrapper::get_page_size;
use ::parser::common::ValueList;
use super::buffer::{DataPtr, PageRef};
use super::table::TableRef;
use super::tuple::tuple_len;


#[derive(Debug)]
pub struct PageHeader {
    pub slot_sum : u32,
    pub first_free_page : u32,
    pub data : DataPtr,
}

impl PageHeader {
    pub fn save_to_page_data(&mut self) {
        unsafe{
            write::<u32>(self.data as *mut u32, self.slot_sum);
            let next_data_ptr = self.data.offset(size_of::<u32>() as isize);
            write::<u32>(next_data_ptr as *mut u32, self.first_free_page);
        }
    }
    pub fn init_from_page_data(&mut self) {
        unsafe{
            self.slot_sum = read::<u32>(self.data as *const u32);
            let next_data_ptr = self.data.offset(size_of::<u32>() as isize);
            self.first_free_page = read::<u32>(next_data_ptr as *const u32);
        }
    }
}

#[derive(Debug)]
pub struct BitMap {
    data : DataPtr,
    size : usize,
}

impl BitMap {
    pub fn clean(&mut self) {
        unsafe{
            write_bytes(self.data, 0, self.size);
        }
    }
}

#[derive(Debug)]
pub struct FilePage {
    header : PageHeader,
    bitmap : BitMap,
    tuple_data : DataPtr,
    mem_page : PageRef,
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
                slot_sum : slot_sum as u32,
                first_free_page : 0,
                data : data,
            },
            bitmap : BitMap{
                data : bitmap_data,
                size : bitmap_size,
            },
            tuple_data : tuple_data,
            mem_page : mem_page,
        }
    }
    pub fn init_empty_page(&mut self) {
        self.header.save_to_page_data();
        self.bitmap.clean();
    }
    pub fn init_from_data(&mut self) {}
}


type TableFileRef = Arc<RwLock<TableFile>>;

#[derive(Debug)]
pub struct TableFile {
    saved_name : String,
    // fd : i32,
    file : File,
    page_list : Vec<FilePage>,
    table : TableRef,
    first_free_page : u32,
    tuple_len : usize,  // for FilePage
}

impl TableFile {
    pub fn new(name : &String, table : TableRef) -> TableFile {
        let file = OpenOptions::new().read(true).write(true).create(true).open(name).unwrap();
        let tuple_len = tuple_len(&table.read().unwrap().attr_list);
        TableFile{
            saved_name : name.clone(),
            file : file,
            page_list : Vec::new(),
            table : table,
            first_free_page : 0,
            tuple_len : tuple_len,
        }
    }
    pub fn init_from_file(&mut self) {}
    pub fn insert(&mut self, _value_list : &ValueList) {}
}


#[derive(Debug)]
pub struct TableFileManager {
    files : HashMap<String, TableFileRef>,  // key is table name
}

impl TableFileManager {
    fn new() -> TableFileManager {
        TableFileManager{ files : HashMap::new() }
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
