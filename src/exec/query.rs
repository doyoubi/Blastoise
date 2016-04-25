use std::boxed::Box;
use std::option::Option;
use std::collections::HashSet;
use ::store::table::{TableManagerRef, IndexMap};
use ::store::tuple::{TupleData, TupleDesc};
use ::store::file::TableFileRef;
use ::store::buffer::PageKey;
use ::parser::condition::CondRef;
use super::iter::{ExecIter, ExecIterRef};
use super::error::ExecError;
use super::evaluate::PtrMap;
use super::evaluate::eval_cond;


#[derive(Debug)]
pub struct FileScan {
    table : String,
    table_manager : TableManagerRef,
    curr_position : usize,
    pinned_pages : HashSet<PageKey>,
    file : TableFileRef,
    finished : bool,
}

impl FileScan {
    pub fn new(table : &String, table_manager : &TableManagerRef) -> ExecIterRef {
        let file = table_manager.borrow_mut().file_manager.get_file(&table);
        Box::new(FileScan{
            table : table.clone(),
            table_manager : table_manager.clone(),
            curr_position : 0,
            pinned_pages : HashSet::new(),
            file : file,
            finished : false,
        })
    }
    fn find_page_helper(&mut self, page_index : &mut usize,
            tuple_index : &mut usize) -> Option<usize> {
        let page_sum = self.file.borrow().page_sum;
        let slot_sum = self.file.borrow().get_page_slot_sum();
        while *page_index < page_sum {
            let next = self.file.borrow().next_tuple_index(*page_index, *tuple_index);
            match next {
                Some(i) => return Some(*page_index * slot_sum + i),
                None => {
                    let fd = self.file.borrow().get_fd();
                    self.pinned_pages.remove(&PageKey{ fd : fd, page_index : *page_index as u32 });
                    self.table_manager.borrow_mut().file_manager.unpin_page(fd, *page_index as u32);
                    *page_index += 1;
                    *tuple_index = 0;
                    if *page_index < page_sum {
                        self.table_manager.borrow_mut().file_manager.ensure_page_loaded(
                            &self.file, *page_index);
                        self.table_manager.borrow_mut().file_manager.pin_page(fd, *page_index as u32);
                        self.pinned_pages.insert(PageKey{ fd : fd, page_index : *page_index as u32 });
                    }
                }
            }
        }
        None
    }
}

impl ExecIter for FileScan {
    fn open(&mut self) {
        assert!(!self.finished);
        let page_num = self.file.borrow().page_sum;
        if page_num == 0 {
            self.close();
            return;
        }
        let fd = self.file.borrow().get_fd();
        self.pinned_pages.insert(PageKey{ fd : fd, page_index : 0 });
        let mut table_manager = self.table_manager.borrow_mut();
        table_manager.file_manager.ensure_page_loaded(&self.file, 0);
        table_manager.file_manager.pin_page(fd, 0);
    }
    fn close(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let mut table_manager = self.table_manager.borrow_mut();
        for &PageKey{ fd, page_index } in self.pinned_pages.iter() {
            table_manager.file_manager.unpin_page(fd, page_index);
        }
    }
    fn explain(&self) -> String {
        format!("file scan, page sum: {:?}",
            self.file.borrow().page_sum)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        let file = self.file.clone();
        let slot_sum = file.borrow().get_page_slot_sum();
        let shift_index = if self.curr_position == 0 {
            self.curr_position
        } else {
            self.curr_position - 1
        };  // to stay in the same page as the last get_next()
        let mut page_index = shift_index / slot_sum;
        let mut tuple_index = self.curr_position - slot_sum * page_index;
        let index = self.find_page_helper(&mut page_index, &mut tuple_index);
        let result = match index {
            Some(position) => Some((
                self.table_manager.borrow_mut().file_manager.get_tuple_data(
                    &self.table, position).unwrap(),
                position
            )),
            None => None,
        };
        match result {
            Some((tuple_data, new_position)) => {
                self.curr_position = new_position + 1;
                Some(tuple_data)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
    fn get_error(&self) -> Option<ExecError> { None }
}


#[derive(Debug)]
pub struct Filter {
    data_source : ExecIterRef,
    condition : CondRef,
    index_map : IndexMap,
    tuple_desc : TupleDesc,
    finished : bool,
}

impl Filter {
    pub fn new(
            condition : CondRef,
            index_map : IndexMap,
            tuple_desc : TupleDesc,
            inner_iter : ExecIterRef) -> ExecIterRef {
        Box::new(Filter{
            condition : condition,
            data_source : inner_iter,
            index_map : index_map,
            tuple_desc : tuple_desc,
            finished : false,
        })
    }
}

impl ExecIter for Filter {
    fn open(&mut self) {
        self.data_source.open();
    }
    fn close(&mut self) {
        self.data_source.close();
        self.finished = true;
    }
    fn explain(&self) -> String {
        format!("filtered by condition: {:?} from source {:?}", self.condition, self.data_source)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        assert_eq!(self.index_map.len(), self.tuple_desc.attr_desc.len());
        while let Some(tuple_data) = self.data_source.get_next() {
            assert_eq!(self.index_map.len(), tuple_data.len());
            let mut ptr_map = PtrMap::new();
            for (k, index) in &self.index_map {
                ptr_map.insert(k.clone(), (
                    tuple_data[*index],
                    self.tuple_desc.attr_desc[*index].clone()
                    ));
            }
            if eval_cond(&*self.condition, &ptr_map) {
                return Some(tuple_data);
            }
        }
        self.close();
        None
    }
    fn get_error(&self) -> Option<ExecError> { None }
}


#[derive(Debug)]
pub struct Projection {
    data_source : ExecIterRef,
    proj_attr_index : Vec<usize>,
    proj_attr_list : Vec<(String, String)>,
    finished : bool,
}

impl Projection {
    pub fn new(
            attr_index : Vec<usize>,
            proj_attr_list : Vec<(String, String)>,
            inner_iter : ExecIterRef) -> ExecIterRef {
        Box::new(Projection{
            data_source : inner_iter,
            proj_attr_index : attr_index,
            proj_attr_list : proj_attr_list,
            finished : false,
        })
    }
}

impl ExecIter for Projection {
    fn open(&mut self) {
        self.data_source.open();
    }
    fn close(&mut self) {
        self.data_source.close();
        self.finished = true;
    }
    fn explain(&self) -> String {
        format!("Projection: {:?} from source {:?}", self.proj_attr_list, self.data_source)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        match self.data_source.get_next() {
            Some(tuple_data) => {
                let mut result = Vec::new();
                for i in &self.proj_attr_index {
                    result.push(tuple_data[*i]);
                }
                Some(result)
            }
            None => {
                self.close();
                None
            }
        }
    }
    fn get_error(&self) -> Option<ExecError> { None }
}
