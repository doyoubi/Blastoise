use std::vec::Vec;
use std::boxed::Box;
use std::collections::HashMap;
use ::parser::common::{Statement, ValueExpr, ValueType};
use ::parser::select::{Relation, SelectExpr};
use ::parser::attribute::AttributeExpr;
use ::parser::condition::gen_check_primary_key_condition_expr;
use ::parser::{
    SelectStatement,
    InsertStatement,
    UpdateStatement,
    DeleteStatement,
    CreateStatement,
    DropStatement,
};
use ::store::table::{TableSet, TableManagerRef, TableRef};
use ::store::tuple::TupleValue; 
use super::iter::ExecIterRef;
use super::create_drop::{CreateTable, DropTable};
use super::change::{Insert, CheckAndInsert, Update, Delete};
use super::query::{FileScan, Filter, Projection};


pub fn gen_plan(stmt : Statement, table_manager : &TableManagerRef)
        -> ExecIterRef {
    match stmt {
        Statement::Create(create) => gen_create_plan(create, table_manager),
        Statement::Drop(drop) => gen_drop_plan(drop, table_manager),
        Statement::Insert(insert) => gen_insert_plan(insert, table_manager),
        Statement::Update(update) => gen_update_plan(update, table_manager),
        Statement::Delete(delete) => gen_delete_plan(delete, table_manager),
        Statement::Select(select) => gen_select_plan(select, table_manager),
    }
}

pub fn gen_create_plan(stmt : CreateStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    CreateTable::new(stmt, table_manager)
}

pub fn gen_drop_plan(stmt : DropStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    DropTable::new(stmt, table_manager)
}

pub fn gen_select_plan(stmt : SelectStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    // join and sub query not supported now
    let table_name = extract!(&stmt.relation_list[0], &Relation::TableName(ref name), name.clone());
    let table = table_manager.borrow().get_table(&table_name).unwrap();
    let mut query = FileScan::new(&table_name, table_manager);
    let (attr_index, proj_attr_list) = gen_select_proj_info(&stmt, &table);
    let need_proj = is_match!(stmt.select_expr, SelectExpr::AttrList(..));
    if let Some(cond) = stmt.where_condition {
        query = Filter::new(Box::new(cond),
            table.borrow().gen_index_map(),
            table.borrow().gen_tuple_desc(), query);
    }
    if need_proj {
        query = Projection::new(attr_index, proj_attr_list, query);
    }
    query
}

pub fn gen_select_proj_info(
        stmt : &SelectStatement, table : &TableRef) -> (Vec<usize>, Vec<(String, String)>) {
    let table = table.borrow();
    let mut proj_attr_index = Vec::new();
    let mut proj_attr_list = Vec::new();
    let mut table_and_attr_list = match stmt.select_expr {
        SelectExpr::AttrList(ref l) => {
            let mut table_and_attr_list = Vec::new();
            for attr in l {
                let table_and_attr = extract!(attr, &AttributeExpr::TableAttr{ref table, ref attr},
                    (table.clone().unwrap(), attr.clone()));
                table_and_attr_list.push(table_and_attr);
            }
            table_and_attr_list
        }
        SelectExpr::AllAttribute => {
            table.get_attr_name_list().iter().map(|a| (table.name.clone(), a.clone())).collect()
        }
    };
    let index_map = table.gen_index_map();
    for table_and_attr in table_and_attr_list.drain(..) {
        proj_attr_index.push(index_map.get(&table_and_attr).unwrap().clone());
        proj_attr_list.push(table_and_attr);
    }
    (proj_attr_index, proj_attr_list)
}

pub fn gen_proj_info(
        stmt : &Statement, table_manager : &TableManagerRef) -> (Vec<usize>, Vec<(String, String)>) {
    let mut proj_attr_index = Vec::new();
    let mut proj_attr_list = Vec::new();
    let table = get_stmt_table(stmt, table_manager);
    if let &Statement::Select(ref select) = stmt {
        return gen_select_proj_info(select, &table);
    } else {
        let table = table.borrow();
        let table_name = table.name.clone();
        for (i, attr) in table.attr_list.iter().enumerate() {
            proj_attr_list.push((table_name.clone(), attr.name.clone()));
            proj_attr_index.push(i);
        }
    }
    (proj_attr_index, proj_attr_list)
}

pub fn get_stmt_table(stmt : &Statement, table_manager : &TableManagerRef) -> TableRef {
    match stmt {
        &Statement::Create(..) | &Statement::Drop(..) => panic!("invalid state"),
        &Statement::Insert(ref insert) => table_manager.borrow().get_table(&insert.table).unwrap(),
        &Statement::Update(ref update) => table_manager.borrow().get_table(&update.table).unwrap(),
        &Statement::Delete(ref delete) => table_manager.borrow().get_table(&delete.table).unwrap(),
        &Statement::Select(ref select) => {
            let table_name = extract!(
                select.relation_list[0], Relation::TableName(ref name), name);
            table_manager.borrow().get_table(&table_name).unwrap()
        }
    }
}

pub fn gen_delete_plan(stmt : DeleteStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    let table = table_manager.borrow().get_table(&stmt.table).unwrap();
    let mut data_source = FileScan::new(&stmt.table, table_manager);
    if let Some(cond) = stmt.where_condition {
        data_source = Filter::new(Box::new(cond),
            table.borrow().gen_index_map(),
            table.borrow().gen_tuple_desc(), data_source);
    }
    Delete::new(&stmt.table, data_source, table_manager)
}

pub fn gen_insert_plan(stmt : InsertStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    let table = table_manager.borrow().get_table(&stmt.table).unwrap();
    let pk_index = table.borrow().get_primary_key_index();
    let pk = stmt.value_list[pk_index].value.parse::<i32>().unwrap();
    let check = gen_check_primary_key_exist_plan(pk, &stmt.table, table_manager);
    CheckAndInsert::new(check, Insert::new(stmt, table_manager))
}

pub fn gen_update_plan(stmt : UpdateStatement, table_manager : &TableManagerRef) -> ExecIterRef {
    let table = table_manager.borrow().get_table(&stmt.table).unwrap();
    let mut data_source = FileScan::new(&stmt.table, table_manager);
    if let Some(cond) = stmt.where_condition {
        data_source = Filter::new(Box::new(cond),
            table.borrow().gen_index_map(),
            table.borrow().gen_tuple_desc(), data_source);
    }
    let mut set_values = HashMap::new();
    let index_map = table.borrow().gen_index_map();
    for assign in stmt.set_list.iter() {
        let attr = &assign.attr;
        let value = &assign.value;
        let index = index_map.get(&(stmt.table.clone(), attr.clone())).unwrap();
        let tuple_value = value_expr_to_tuple_value(value);
        set_values.insert(*index, tuple_value);
    }
    let tuple_desc = table.borrow().gen_tuple_desc();
    Update::new(&stmt.table, tuple_desc, set_values, data_source, table_manager)
}

pub fn value_expr_to_tuple_value(expr : &ValueExpr) -> TupleValue {
    match expr.value_type {
        ValueType::Integer => TupleValue::Int(expr.value.parse::<i32>().unwrap()),
        ValueType::Float => TupleValue::Float(expr.value.parse::<f32>().unwrap()),
        ValueType::String => TupleValue::Char(expr.value.clone()),
        ValueType::Null => unimplemented!(),
    }
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
