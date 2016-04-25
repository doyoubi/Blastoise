use std::collections::HashMap;
use std::mem::size_of;
use std::ptr::{write, read, write_bytes, null_mut};
use std::fs::{OpenOptions, File};
use std::os::unix::io::AsRawFd;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::io::{Read, Write, Seek, SeekFrom};
use std::rc::Rc;
use std::cell::RefCell;
use ::utils::libwrapper::get_page_size;
use ::utils::pointer::{read_string, write_string, pointer_offset};
use ::utils::config::Config;
use ::utils::file::{path_join, ensure_dir_exist, assert_file_exist};
use ::parser::common::{ValueList, ValueType};
use super::buffer::{DataPtr, PageRef, PagePool};
use super::table::{TableRef, AttrType, IndexMap};
use super::tuple::{TupleDesc, TupleValue, TupleData};


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
    pub fn next_tuple_index(&self, from : usize) -> usize {
        if from >= self.slot_sum {
            return self.slot_sum;
        }
        let mut count = from / 8;
        let mut bit_count = from % 8;
        while count < (self.slot_sum + 7) / 8 {
            let n = unsafe{ read::<u8>((self.data as *const u8).offset(count as isize)) };
            let mut mask = 1 << bit_count;
            if n < mask { count += 1; bit_count = 0; continue; }
            loop {
                // assume the bits whose index > slot_sum is 0
                if mask & n > 0 {
                    return count * 8 + bit_count;
                }
                mask *= 2;
                bit_count += 1;
            }
        }
        self.slot_sum
    }
    pub fn get_first_free_slot(&self) -> usize {
        let mut count = 0;
        while count < (self.slot_sum + 7) / 8 {
            let n = unsafe{ read::<u8>((self.data as *const u8).offset(count as isize)) };
            if n == 255 { count += 1; continue; }
            let mut mask : u8 = 1;
            let mut bit_count = 0;
            loop {
                // assume the bits whose index > slot_sum is 0
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
    pub tuple_len : usize,
}

impl FilePage {
    pub fn new(mem_page : PageRef, tuple_len : usize) -> FilePage {
        let data = mem_page.borrow_mut().data;
        let header_size = 2 * size_of::<u32>();  // PageHeader
        let slot_sum = get_slot_sum(tuple_len);
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
            tuple_len : tuple_len,
        }
    }
    pub fn init_empty_page(&mut self) {
        self.header.save_to_page_data();
        self.bitmap.clean();
    }
    pub fn init_from_page_data(&mut self) {
        self.header.init_from_page_data();
    }
    pub fn save_to_page(&mut self) {
        self.header.save_to_page_data();
    }
    pub fn is_inuse(&self, index : usize) -> bool {
        self.bitmap.is_inuse(index)
    }
    pub fn set_inuse(&mut self, index : usize, inuse : bool) {
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
                    unsafe{ write_string(p, &v.value, len) };
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
    pub fn get_tuple_value(&self, tuple_index : usize,
            attr_position : usize,
            tuple_desc : &TupleDesc) -> TupleValue {
        assert!(self.is_inuse(tuple_index));
        let mut p = pointer_offset(self.tuple_data, tuple_index * tuple_desc.tuple_len);
        p = Self::attr_offset(p, tuple_desc, attr_position);
        unsafe{
            match tuple_desc.attr_desc[attr_position] {
                AttrType::Int => TupleValue::Int(read::<i32>(p as *const i32)),
                AttrType::Float => TupleValue::Float(read::<f32>(p as *const f32)),
                AttrType::Char{len} => TupleValue::Char(read_string(p, len)),
            }
        }
    }
    pub fn get_tuple_data(&self, tuple_index : usize, tuple_desc : &TupleDesc) -> Option<TupleData> {
        if tuple_index >= self.bitmap.slot_sum {
            return None;
        }
        assert!(self.is_inuse(tuple_index));
        let mut tuple_data = Vec::new();
        let data = pointer_offset(self.tuple_data, tuple_index * tuple_desc.tuple_len);
        let mut p = data;
        for i in 1..tuple_desc.attr_desc.len() + 1 {
            tuple_data.push(p);
            p = Self::attr_offset(data, tuple_desc, i);
        }
        Some(tuple_data)
    }
    pub fn attr_offset(p : DataPtr, tuple_desc : &TupleDesc, attr_position : usize) -> DataPtr {
        let mut offset = 0;
        for (attr_type, _) in tuple_desc.attr_desc.iter().zip(0..attr_position) {
            match attr_type {
                &AttrType::Int | &AttrType::Float => offset += 4,
                &AttrType::Char{len} => offset += (len + 3) / 4 * 4,
            }
        }
        pointer_offset(p, offset)
    }
    pub fn is_full(&self) -> bool {
        self.header.first_free_slot == self.bitmap.slot_sum
    }
    pub fn is_in_page(&self, ptr : DataPtr) -> bool {
        let page_start = self.mem_page.borrow().data;
        let page_end = pointer_offset(page_start, get_page_size());
        page_start <= ptr && ptr < page_end
    }
    pub fn delete(&mut self, ptr : DataPtr) {
        let d = ptr as usize - self.tuple_data as usize;
        let index = d / self.tuple_len;
        assert!(self.is_inuse(index));
        self.set_inuse(index, false);
    }
}


pub type TableFileRef = Rc<RefCell<TableFile>>;

#[derive(Debug)]
pub struct TableFile {
    pub saved_name : String,
    pub file : File,
    pub loaded_pages : HashMap<usize, FilePage>,
    pub page_sum : usize,  // including pages not loaded in memory
    pub table : TableRef,
    pub first_free_page : usize,
    pub tuple_desc : TupleDesc,  // for FilePage
}

impl TableFile {
    pub fn new(mut name : String, table : TableRef, dir : &String) -> TableFile {
        name.push_str(".table");
        name = path_join(dir, &name);
        let file = OpenOptions::new().read(true).write(true).create(true).open(&name).unwrap();
        let tuple_desc = table.borrow().gen_tuple_desc();
        TableFile{
            saved_name : name,
            file : file,
            loaded_pages : HashMap::new(),
            page_sum : 0,
            table : table,
            first_free_page : 0,
            tuple_desc : tuple_desc,
        }
    }
    pub fn init_from_file(&mut self) {
        is_match!(self.file.seek(SeekFrom::Start(0)), Ok(..));
        let mut header = [0 as u32, 0 as u32];
        is_match!(self.file.read_exact(unsafe{
            from_raw_parts_mut::<u8>((&mut header).as_ptr() as *mut u8, 8)
        }), Ok(..));
        self.page_sum = header[0] as usize;
        self.first_free_page = header[1] as usize;
    }
    pub fn read_page_from_file(&mut self, data : DataPtr, page_index : usize) {
        assert!(page_index < self.page_sum);
        let page_size = get_page_size();
        let offset = page_size * (page_index + 1);
        is_match!(self.file.seek(SeekFrom::Start(offset as u64)), Ok(..));
        is_match!(self.file.read_exact(unsafe{
            from_raw_parts_mut::<u8>(data as *mut u8, page_size)
        }), Ok(..));
    }
    pub fn get_page_slot_sum(&self) -> usize {
        get_slot_sum(self.tuple_desc.tuple_len)
    }
    pub fn save_to_file(&mut self) {
        // the first page only save header for alignment
        is_match!(self.file.seek(SeekFrom::Start(0)), Ok(..));
        let header = [self.page_sum as u32, self.first_free_page as u32];
        is_match!(self.file.write_all(unsafe{
            from_raw_parts::<u8>((&header).as_ptr() as *const u8, 8)
        }), Ok(..));
        let index_list : Vec<_> = self.loaded_pages.iter().map(|(i, _)| *i).collect();
        for i in index_list.iter() {
            self.save_page(*i);
        }
    }
    pub fn save_page(&mut self, page_index : usize) {
        // the first page only save header for alignment
        let page_size = get_page_size();
        let offset = page_size * (page_index + 1);
        let page = self.loaded_pages.get(&page_index).unwrap();
        is_match!(self.file.seek(SeekFrom::Start(offset as u64)), Ok(..));
        is_match!(self.file.write_all(unsafe{
            from_raw_parts::<u8>(page.mem_page.borrow().data as *const u8, page_size)
        }), Ok(..));
    }
    pub fn delete(&mut self, ptr : DataPtr) {
        for (_, page) in &mut self.loaded_pages {
            if page.is_in_page(ptr) {
                page.delete(ptr);
                return;
            }
        }
    }
    pub fn insert(&mut self, value_list : &ValueList) {
        // must call add_page first if need_new_page() is true
        let first_free_page = self.first_free_page;
        self.insert_in_page(first_free_page, value_list)
    }
    pub fn insert_in_page(&mut self, page_index : usize, value_list : &ValueList) {
        // for test
        assert!(page_index < self.page_sum);
        let file_page = self.loaded_pages.get_mut(&page_index).unwrap();
        assert!(!file_page.is_full());
        file_page.insert(value_list, &self.tuple_desc)
    }
    pub fn get_tuple_value(&self, position : usize, attr_position : usize) -> TupleValue {
        // only for test
        let page_index = position / self.get_page_slot_sum();
        let tuple_index = position % self.get_page_slot_sum();
        assert!(self.loaded_pages.get(&page_index).is_some());
        let page = self.loaded_pages.get(&page_index).unwrap();
        page.get_tuple_value(tuple_index, attr_position, &self.tuple_desc)
    }
    pub fn get_tuple_data(&self, position : usize) -> Option<TupleData> {
        let page_index = position / self.get_page_slot_sum();
        let tuple_index = position % self.get_page_slot_sum();
        assert!(self.loaded_pages.get(&page_index).is_some());
        let page = self.loaded_pages.get(&page_index).unwrap();
        page.get_tuple_data(tuple_index, &self.tuple_desc)
    }
    pub fn next_tuple_index(&self, page_index : usize, tuple_index : usize) -> Option<usize> {
        assert!(self.loaded_pages.get(&page_index).is_some());
        let page = self.loaded_pages.get(&page_index).unwrap();
        let next = page.bitmap.next_tuple_index(tuple_index);
        if next == self.get_page_slot_sum() {
            None
        } else {
            Some(next)
        }
    }
    pub fn add_page(&mut self, mem_page : PageRef) {
        let file_page = FilePage::new(mem_page, self.tuple_desc.tuple_len);
        let index = file_page.mem_page.borrow().page_index as usize;
        self.loaded_pages.insert(index, file_page);
    }
    pub fn get_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
    pub fn is_inuse(&self, page_index : usize, tuple_index : usize) -> bool {
        assert!(self.loaded_pages.get(&page_index).is_some());
        let page = self.loaded_pages.get(&page_index).unwrap();
        page.is_inuse(tuple_index)
    }
    pub fn gen_index_map(&self) -> IndexMap {
        self.table.borrow().gen_index_map()
    }
}


#[derive(Debug)]
pub struct TableFileManager {
    files : HashMap<String, TableFileRef>,  // key is table name
    pub page_pool : PagePool,
    table_file_dir : String,
}

impl TableFileManager {
    pub fn new(config : &Config) -> TableFileManager {
        let table_file_dir = config.get_str("table_file_dir");
        ensure_dir_exist(&table_file_dir);
        TableFileManager{
            files : HashMap::new(),
            page_pool : PagePool::new(config.get_int("max_memory_pool_page_num") as usize),
            table_file_dir : table_file_dir,
        }
    }
    pub fn init_from_file(&mut self, tables : Vec<TableRef>) {
        for table in &tables {
            let table_name = table.borrow().name.clone();
            let mut file_name = table_name.clone();
            file_name.push_str(".table");
            let full_path = path_join(&self.table_file_dir, &file_name);
            assert_file_exist(&full_path);
            self.create_file(table_name.clone(), table.clone());
            self.files.get_mut(&table_name).unwrap().borrow_mut().init_from_file();
        }
    }
    pub fn save_all(&mut self) {
        for (_, f)  in self.files.iter() {
            f.borrow_mut().save_to_file();
        }
    }
    pub fn delete(&mut self, table : &String, ptr : DataPtr) {
        let file = self.get_file(table);
        file.borrow_mut().delete(ptr);
    }
    pub fn insert(&mut self, table : &String, value_list : &ValueList) {
        let file = self.get_file(table);
        let is_new_page = self.need_new_page(&file);  // fight the borrow checker, RefCell
        if is_new_page {
            let new_page_index = file.borrow().page_sum;
            self.ensure_page_loaded(&file, new_page_index);
            file.borrow_mut().loaded_pages.get_mut(&new_page_index).unwrap().init_empty_page();
        } else {
            let first_free_page = file.borrow().first_free_page;
            self.ensure_page_loaded(&file, first_free_page);
        }
        file.borrow_mut().insert(value_list);
    }
    pub fn insert_in_page(&mut self, table : &String, page_index : usize, value_list : &ValueList) {
        // for test
        self.prepare_page(table, page_index);
        let file = self.get_file(table);
        file.borrow_mut().insert_in_page(page_index, value_list);
    }
    pub fn prepare_page(&mut self, table : &String, page_index : usize) {
        // for test, will init empty page
        let file = self.get_file(&table);
        let page_exist = file.borrow().loaded_pages.get(&page_index).is_some();  // fight borrow checker
        if !page_exist {
            self.ensure_page_loaded(&file, page_index);
            file.borrow_mut().loaded_pages.get_mut(&page_index).unwrap().init_empty_page();
        }
    }
    pub fn need_new_page(&mut self, file : &TableFileRef) -> bool {
        let page_sum = file.borrow().page_sum;
        let mut first_free_page;
        loop {
            first_free_page = file.borrow().first_free_page;
            assert!(first_free_page <= page_sum);
            if first_free_page == page_sum { break; }
            self.ensure_page_loaded(&file, first_free_page);
            let is_full = file.borrow().loaded_pages.get(&first_free_page).unwrap().is_full();  // fight borrow checker
            if is_full {
                file.borrow_mut().first_free_page += 1;
            } else {
                return false;
            }
        }
        true
    }
    pub fn get_file(&mut self, table : &String) -> TableFileRef {
        self.files.get_mut(table).unwrap().clone()
    }
    pub fn get_tuple_value(&mut self, table : &String,
            position : usize,
            attr_position : usize) -> TupleValue{
        // only for test
        let file = self.files.get(table).unwrap().clone();
        let page_index = {
            let f = file.borrow_mut();
            position / f.tuple_desc.tuple_len
        };
        self.prepare_page(table, page_index);
        // declare v only to fight lifetime checker
        let v = file.borrow().get_tuple_value(position, attr_position);
        v
    }
    pub fn get_tuple_data(&mut self, table : &String, position : usize) -> Option<TupleData> {
        let file = self.files.get(table).unwrap().clone();
        let page_index = {
            let f = file.borrow_mut();
            position / f.get_page_slot_sum()
        };
        self.ensure_page_loaded(&file, page_index);
        // declare v only to fight lifetime checker
        let v = file.borrow().get_tuple_data(position);
        v
    }
    pub fn get_next_tuple_data(&mut self, table : &String, from : usize) -> Option<(TupleData, usize)> {
        match self.get_next_position(table, from) {
            Some(position) => Some((self.get_tuple_data(table, position).unwrap(), position)),
            None => None,
        }
    }
    pub fn get_next_position(&mut self, table : &String, from : usize) -> Option<usize> {
        let file = self.get_file(table);
        let page_sum = file.borrow().page_sum;
        let slot_sum = file.borrow().get_page_slot_sum();
        let mut page_index = from / slot_sum;
        let mut tuple_index = from % slot_sum;
        while page_index < page_sum {
            let next = file.borrow().next_tuple_index(page_index, tuple_index);
            match next {
                Some(i) => return Some(page_index * slot_sum + i),
                None => {
                   page_index += 1;
                   tuple_index = 0;
                }
            }
        }
        None
    }
    pub fn ensure_page_loaded(&mut self, file : &TableFileRef, page_index : usize) {
        let page_sum = file.borrow().page_sum;
        assert!(page_index < page_sum || page_index == page_sum);  // old page or new page
        let page_exist = file.borrow().loaded_pages.get(&page_index).is_some();  // fight borrow checker
        if !page_exist {
            let fd = file.borrow().get_fd();
            let mut ptr = null_mut();
            if let Some(page) = self.page_pool.prepare_page() {
                // save tail page
                let old_page_index = page.borrow().page_index;
                ptr = page.borrow().data;
                let old_fd = page.borrow().fd;
                let old_file = self.get_file_by_fd(old_fd);
                old_file.borrow_mut().save_page(old_page_index as usize);
                page.borrow_mut().data = null_mut();
                old_file.borrow_mut().loaded_pages.remove(&(old_page_index as usize));
                self.page_pool.remove_tail();
            }
            self.page_pool.put_page(fd, page_index as u32, ptr);
            {
                let page = self.page_pool.get_page(fd, page_index as u32).unwrap();
                ptr = page.borrow().data.clone();
            }
            if page_index < page_sum {
                file.borrow_mut().read_page_from_file(ptr, page_index);
                file.borrow_mut().add_page(self.page_pool.get_page(fd, page_index as u32).unwrap());
                file.borrow_mut().loaded_pages.get_mut(&page_index).unwrap().init_from_page_data();
            } else {
                file.borrow_mut().page_sum += 1;
                file.borrow_mut().add_page(self.page_pool.get_page(fd, page_index as u32).unwrap());
            }
        }
    }
    pub fn get_file_by_fd(&self, fd : i32) -> TableFileRef {
        for (_, file) in self.files.iter() {
            if file.borrow().get_fd() == fd {
                return file.clone();
            }
        }
        panic!("invalid fd");
    }
    pub fn create_file(&mut self, name : String, table : TableRef) {
        let file = TableFile::new(name.clone(), table, &self.table_file_dir);
        self.files.insert(name, Rc::new(RefCell::new(file)));
    }
    pub fn pin_page(&mut self, fd : i32, page_index : u32) {
        self.page_pool.pin_page(fd, page_index);
    }
    pub fn unpin_page(&mut self, fd : i32, page_index : u32) {
        self.page_pool.unpin_page(fd, page_index);
    }
    pub fn get_unpinned_num(&self) -> usize {
        self.page_pool.get_unpinned_num()
    }
    pub fn get_file_fd(&self, name : &String) -> i32 {
        self.files.get(name).unwrap().borrow().get_fd()
    }
}

fn get_slot_sum(tuple_len : usize) -> usize {
    let header_size = 2 * size_of::<u32>();  // PageHeader
    let page_size = get_page_size();
    // (n + 8 - 1) / 8 + tuple_len * n <= page_size - header_size
    (8 * (page_size - header_size) - 7) / (8 * tuple_len + 1)
}
