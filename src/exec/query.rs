use std::boxed::Box;
use std::option::Option;
use ::store::table::TableManagerRef;
use ::store::tuple::{TupleData, TupleDesc};
use ::store::table::IndexMap;
use ::parser::condition::CondRef;
use super::iter::{ExecIter, ExecIterRef};
use super::evaluate::PtrMap;
use super::evaluate::eval_cond;


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
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("filtered by condition: {:?}", self.condition)
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
            index_map : &IndexMap,
            proj_attr_list : Vec<(String, String)>,
            inner_iter : ExecIterRef) -> ExecIterRef {
        let mut proj_attr_index = Vec::new();
        for k in proj_attr_list.iter() {
            proj_attr_index.push(index_map.get(&k).unwrap().clone());
        }
        Box::new(Projection{
            data_source : inner_iter,
            proj_attr_index : proj_attr_index,
            proj_attr_list : proj_attr_list,
            finished : false,
        })
    }
}

impl ExecIter for Projection {
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("Projection: {:?}", self.proj_attr_list)
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
}
