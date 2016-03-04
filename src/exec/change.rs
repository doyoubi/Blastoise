use std::boxed::Box;
use std::option::Option;
use ::store::table::TableManagerRef;
use ::store::tuple::TupleData;
use ::parser::InsertStatement;
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
    fn open(&mut self) {}
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
