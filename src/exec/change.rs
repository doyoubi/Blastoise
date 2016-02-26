use std::boxed::Box;
use std::option::Option;
use ::store::table::{Table, Attr, AttrType, TableManagerRef};
use ::store::tuple::TupleData;
use ::parser::InsertStatement;
use ::parser;
use super::iter::{ExecIter, ExecIterRef};


#[derive(Debug)]
struct Insert {
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
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("{}", self.stmt)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        None
    }
}
