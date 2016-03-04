use std::vec::Vec;
use ::parser::common::Statement;
use ::parser::select::Relation;
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
use super::error::ExecError;
use super::create_drop::{CreateTable, DropTable};
use super::change::Insert;


pub type PlanResult = Result<ExecIterRef, ExecError>;

pub fn gen_plan(stmt : Statement, table_manager : &TableManagerRef, _table_set : TableSet)
        -> PlanResult {
    match stmt {
        Statement::Create(create) => gen_create_plan(create, table_manager),
        Statement::Drop(drop) => gen_drop_plan(drop, table_manager),
        Statement::Insert(insert) => gen_insert_plan(insert, table_manager),
        _ => unimplemented!(),
    }
}

pub fn gen_create_plan(stmt : CreateStatement, table_manager : &TableManagerRef) -> PlanResult {
    Ok(CreateTable::new(stmt, table_manager))
}

pub fn gen_drop_plan(stmt : DropStatement, table_manager : &TableManagerRef) -> PlanResult {
    Ok(DropTable::new(stmt, table_manager))
}

pub fn gen_insert_plan(stmt : InsertStatement, table_manager : &TableManagerRef) -> PlanResult {
    // TODO: use select to check if primary key already exist
    Ok(Insert::new(stmt, table_manager))
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
