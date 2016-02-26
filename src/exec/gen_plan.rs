use std::collections::HashMap;
use std::vec::Vec;
use std::sync::MutexGuard;
use ::parser::common::Statement;
use ::parser::select::Relation;
use ::parser::sem_check::check_sem;
use ::parser::{
    SelectStatement,
    // InsertStatement,
    // UpdateStatement,
    // DeleteStatement,
    CreateStatement,
    DropStatement,
};
use ::store::table::{TableSet, TableManagerRef, TableManager};
use super::iter::ExecIterRef;
use super::error::ExecError;
use super::create_drop::{CreateTable, DropTable};


pub type PlanResult = Result<ExecIterRef, ExecError>;

pub fn gen_plan(stmt : Statement, table_manager : &TableManagerRef, table_set : TableSet)
        -> PlanResult {
    match stmt {
        Statement::Create(create) => gen_create_plan(create, table_manager),
        Statement::Drop(drop) => gen_drop_plan(drop, table_manager),
        _ => unimplemented!(),
    }
}

pub fn gen_create_plan(stmt : CreateStatement, table_manager : &TableManagerRef) -> PlanResult {
    Ok(CreateTable::new(stmt, table_manager))
}

pub fn gen_drop_plan(stmt : DropStatement, table_manager : &TableManagerRef) -> PlanResult {
    Ok(DropTable::new(stmt, table_manager))
}

pub fn gen_table_set(stmt : &Statement, table_manager : &TableManagerRef) -> TableSet {
    let mut rw_table = HashMap::new();
    match stmt {
        &Statement::Select(ref select) => {
            let mut tables = gen_select_table_set_helper(select);
            for name in tables.drain(..) {
                rw_table.insert(name, false);
            }
        }
        &Statement::Delete(ref delete) =>
            { rw_table.insert(delete.table.clone(), true); }
        &Statement::Update(ref update) =>
            { rw_table.insert(update.table.clone(), true); }
        &Statement::Insert(ref insert) =>
            { rw_table.insert(insert.table.clone(), true); }
        &Statement::Create(ref create) => {
            if let Some(..) = table_manager.lock().unwrap().get_table(&create.table) {
                rw_table.insert(create.table.clone(), false);
            }
        }
        &Statement::Drop(ref drop) => {
            if let Some(..) = table_manager.lock().unwrap().get_table(&drop.table) {
                rw_table.insert(drop.table.clone(), true);
            }
        }

    }
    table_manager.lock().unwrap().gen_table_set(&rw_table)
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
