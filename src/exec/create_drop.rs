use std::boxed::Box;
use std::option::Option;
use ::store::table::{Table, Attr, AttrType, TableManagerRef};
use ::store::tuple::TupleData;
use ::parser::{CreateStatement, DropStatement};
use ::parser;
use super::iter::{ExecIter, ExecIterRef};
use super::error::ExecError;


#[derive(Debug)]
pub struct CreateTable {
    stmt : CreateStatement,
    finished : bool,
    table_manager : TableManagerRef,
}

impl CreateTable {
    pub fn new(stmt : CreateStatement, table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(CreateTable{
            finished : false,
            stmt : stmt,
            table_manager : table_manager.clone(),
        })
    }
}

impl ExecIter for CreateTable {
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("{}", self.stmt)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        let mut attr_list = Vec::new();
        for attr in &self.stmt.decl_list {
            attr_list.push(Attr{
                name : attr.name.clone(),
                attr_type : match attr.attr_type {
                    parser::create_drop::AttrType::Int => AttrType::Int,
                    parser::create_drop::AttrType::Float => AttrType::Float,
                    parser::create_drop::AttrType::Char{ref len} =>
                        AttrType::Char{len : len.parse::<usize>().unwrap()},
                },
                primary : attr.primary,
                nullable : attr.nullable,
            });
        }
        let table = Table{
            name : self.stmt.table.clone(),
            attr_list : attr_list,
        };
        {
            let mut manager = self.table_manager.borrow_mut();
            manager.add_table(table);
        }
        self.finished = true;
        None
    }
    fn get_error(&self) -> Option<ExecError> { None }
}


#[derive(Debug)]
pub struct DropTable {
    stmt : DropStatement,
    finished : bool,
    table_manager : TableManagerRef,
}

impl DropTable {
    pub fn new(stmt : DropStatement, table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(DropTable{
            finished : false,
            stmt : stmt,
            table_manager : table_manager.clone(),
        })
    }
}

impl ExecIter for DropTable {
    fn open(&mut self) {}
    fn close(&mut self) { self.finished = true; }
    fn explain(&self) -> String {
        format!("{}", self.stmt)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finished {
            return None;
        }
        {
            let mut manager = self.table_manager.borrow_mut();
            manager.remove_table(&self.stmt.table);
        }
        self.finished = true;
        None
    }
    fn get_error(&self) -> Option<ExecError> { None }
}
