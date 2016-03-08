use std::boxed::Box;
use std::option::Option;
use std::collections::HashMap;
use std::ptr::write;
use ::utils::pointer::write_string;
use ::store::table::{AttrType, TableManagerRef};
use ::store::tuple::{TupleData, TupleValue, TupleDesc};
use ::parser::{
    InsertStatement,
};
use super::iter::{ExecIter, ExecIterRef};


#[derive(Debug)]
pub struct Insert {
    stmt : InsertStatement,
    table_manager : TableManagerRef,
    finished : bool,
}

impl Insert {
    pub fn new(stmt : InsertStatement, table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(Insert{
            finished : false,
            stmt : stmt,
            table_manager : table_manager.clone(),
        })
    }
}

impl ExecIter for Insert {
    fn open(&mut self) {
        assert!(!self.finished);
    }
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("{}", self.stmt)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        self.table_manager.borrow_mut().insert(&self.stmt.table, &self.stmt.value_list);
        self.close();
        None
    }
}


#[derive(Debug)]
pub struct Delete {
    table : String,
    data_source : ExecIterRef,
    table_manager : TableManagerRef,
    finished : bool,
}

impl Delete {
    pub fn new(table : &String, data_source : ExecIterRef, table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(Delete{
            table : table.clone(),
            data_source : data_source,
            table_manager : table_manager.clone(),
            finished : false,
        })
    }
}

impl ExecIter for Delete {
    fn open(&mut self) {
        assert!(!self.finished);
        self.data_source.open();
    }
    fn close(&mut self) {
        self.data_source.close();
        self.finished = true;
    }
    fn explain(&self) -> String {
        format!("delete tuple from source: {:?}", self.data_source)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        let tuple_data = match self.data_source.get_next() {
            Some(tuple_data) => tuple_data,
            None => {
                self.close();
                return None;
            }
        };
        self.table_manager.borrow_mut().file_manager.delete(&self.table, tuple_data[0]);
        Some(tuple_data)  // only to indicate not finished, the data inside is only for tests
    }
}


#[derive(Debug)]
pub struct Update {
    table : String,
    data_source : ExecIterRef,
    table_manager : TableManagerRef,
    finished : bool,
    set_values : HashMap<usize, TupleValue>,
    tuple_desc : TupleDesc,
}

impl Update {
    pub fn new(
            table : &String,
            tuple_desc : TupleDesc,
            set_values : HashMap<usize, TupleValue>,
            data_source : ExecIterRef,
            table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(Update{
            table : table.clone(),
            tuple_desc : tuple_desc,
            data_source : data_source,
            table_manager : table_manager.clone(),
            finished : false,
            set_values : set_values,
        })
    }
}

impl ExecIter for Update {
    fn open(&mut self) {
        assert!(!self.finished);
        self.data_source.open();
    }
    fn close(&mut self) {
        self.data_source.close();
        self.finished = true;
    }
    fn explain(&self) -> String {
        format!("update tuple from source: {:?}, set {:?}", self.data_source, self.set_values)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        let tuple_data = match self.data_source.get_next() {
            Some(tuple_data) => tuple_data,
            None => {
                self.close();
                return None;
            }
        };
        for (i, v) in self.set_values.iter() {
            let p = tuple_data[*i];
            unsafe {
                match v {
                    &TupleValue::Int(num) => write::<i32>(p as *mut i32, num),
                    &TupleValue::Float(num) => write::<f32>(p as *mut f32, num),
                    &TupleValue::Char(ref s) => {
                        let len = extract!(self.tuple_desc.attr_desc[*i], AttrType::Char{len}, len);
                        write_string(p, s, len);
                    }
                }
            }
        }
        Some(tuple_data)
    }
}
