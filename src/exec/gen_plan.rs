use std::vec::Vec;
use std::boxed::Box;
use ::parser::common::Statement;
use ::parser::select::Relation;
use ::parser::condition::gen_check_primary_key_condition_expr;
use ::parser::{
    SelectStatement,
    InsertStatement,
    // UpdateStatement,
    // DeleteStatement,
    CreateStatement,
    DropStatement,
};
use ::store::table::{TableSet, TableManagerRef};
use super::iter::ExecIterRef;
use super::create_drop::{CreateTable, DropTable};
use super::change::{Insert, CheckAndInsert};
use super::query::{FileScan, Filter};


pub fn gen_plan(stmt : Statement, table_manager : &TableManagerRef, _table_set : TableSet)
        -> ExecIterRef {
    match stmt {
        Statement::Create(create) => gen_create_plan(create, table_manager),
        Statement::Drop(drop) => gen_drop_plan(drop, table_manager),
        Statement::Insert(insert) => gen_insert_plan(insert, table_manager),
        _ => unimplemented!(),
    }
}

pub fn gen_create_plan(stmt : CreateStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    CreateTable::new(stmt, table_manager)
}

pub fn gen_drop_plan(stmt : DropStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    DropTable::new(stmt, table_manager)
}

pub fn gen_insert_plan(stmt : InsertStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    let table = table_manager.borrow().get_table(&stmt.table).unwrap();
    let pk_index = table.borrow().get_primary_key_index();
    let pk = stmt.value_list[pk_index].value.parse::<i32>().unwrap();
    let check = gen_check_primary_key_exist_plan(pk, &stmt.table, table_manager);
    CheckAndInsert::new(check, Insert::new(stmt, table_manager))
}

pub fn gen_check_primary_key_exist_plan(
        pk : i32,
        table_name : &String,
        table_manager : &TableManagerRef) -> ExecIterRef {
    let table = table_manager.borrow().get_table(table_name).unwrap();
    let pk_attr = table.borrow().get_primary_key_attr();
    let cond = gen_check_primary_key_condition_expr(table_name, &pk_attr.name, pk);
    let scan = FileScan::new(table_name, table_manager);
    let filter = Filter::new(Box::new(cond),
        table.borrow().gen_index_map(),
        table.borrow().gen_tuple_desc(), scan);
    filter
}

pub fn gen_table_set(stmt : &Statement, table_manager : &TableManagerRef) -> TableSet {
    let mut table_list = Vec::new();
    match stmt {
        &Statement::Select(ref select) => {
            let mut tables = gen_select_table_set_helper(select);
            for name in tables.drain(..) {
                table_list.push(name);
            }
        }
        &Statement::Delete(ref delete) =>
            { table_list.push(delete.table.clone()); }
        &Statement::Update(ref update) =>
            { table_list.push(update.table.clone()); }
        &Statement::Insert(ref insert) =>
            { table_list.push(insert.table.clone()); }
        &Statement::Create(ref create) => {
            if let Some(..) = table_manager.borrow().get_table(&create.table) {
                table_list.push(create.table.clone());
            }
        }
        &Statement::Drop(ref drop) => {
            if let Some(..) = table_manager.borrow().get_table(&drop.table) {
                table_list.push(drop.table.clone());
            }
        }

    }
    table_manager.borrow().gen_table_set(&table_list)
}

fn gen_select_table_set_helper(stmt : &SelectStatement) -> Vec<String> {
    let mut result = Vec::new();
    for rel in &stmt.relation_list {
        match rel {
            &Relation::TableName(ref name) => result.push(name.clone()),
            &Relation::Select(ref sub_select) =>
                result.extend_from_slice(&gen_select_table_set_helper(sub_select))
        }
    }
    result
}
