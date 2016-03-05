use std::boxed::Box;
use std::option::Option;
use ::store::table::TableManagerRef;
use ::store::tuple::TupleData;
use super::iter::{ExecIter, ExecIterRef};


#[derive(Debug)]
pub struct FileScan {
    table : String,
    table_manager : TableManagerRef,
    curr_position : usize,
    slot_sum : usize,
    page_sum : usize,
    finished : bool,
}

impl FileScan {
    pub fn new(table : &String, table_manager : &TableManagerRef) -> ExecIterRef {
        let file = table_manager.borrow_mut().file_manager.get_file(&table);
        let slot_sum = file.borrow().get_page_slot_sum();
        let page_sum = file.borrow().page_sum;
        Box::new(FileScan{
            table : table.clone(),
            table_manager : table_manager.clone(),
            curr_position : 0,
            slot_sum : slot_sum,
            page_sum : page_sum,
            finished : false,
        })
    }
}

impl ExecIter for FileScan {
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("file scan, page sum: {:?}", self.page_sum)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        let result =
            self.table_manager.borrow_mut().file_manager.get_next_tuple_data(
                &self.table, self.curr_position);
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
}
