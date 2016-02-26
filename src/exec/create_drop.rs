use std::boxed::Box;
use std::option::Option;
use ::store::table::{Table, Attr, AttrType, TableManagerRef};
use ::store::tuple::TupleData;
use ::parser::CreateStatement;
use ::parser;
use super::iter::{ExecIter, ExecIterRef};


#[derive(Debug)]
pub struct CreateTable {
    stmt : CreateStatement,
    finish : bool,
    table_manager : TableManagerRef,
}

impl CreateTable {
    pub fn new(stmt : CreateStatement, table_manager : &TableManagerRef) -> ExecIterRef {
        Box::new(CreateTable{
            finish : false,
            stmt : stmt,
            table_manager : table_manager.clone(),
        })
    }
}

impl ExecIter for CreateTable {
    fn open(&mut self) {}
    fn close(&mut self) { self.finish = true; }
    fn explain(&self) -> String {
        format!("{}", self.stmt)
    }
    fn get_next(&mut self) -> Option<TupleData> {
        if self.finish {
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
            let mut manager = self.table_manager.lock().unwrap();
            manager.add_table(table);
        }
        self.finish = true;
        None
    }
}
