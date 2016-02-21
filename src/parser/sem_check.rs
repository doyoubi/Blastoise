use std::vec::Vec;
use std::collections::HashSet;
use super::lexer::{Token, TokenRef, TokenType};
use super::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};
use super::common::Statement;
use super::select::SelectStatement;
use super::update::UpdateStatement;
use super::insert::InsertStatement;
use super::delete::DeleteStatement;
use super::create_drop::{CreateStatement, DropStatement};
use ::store::table::TableManager;


pub type SemResult = Result<(), ErrorList>;


pub fn check_sem(statement : Statement, manager : &TableManager) -> SemResult {
    match statement {
        Statement::Select(ref stmt) => check_select(stmt, manager),
        Statement::Update(ref stmt) => check_update(stmt, manager),
        Statement::Insert(ref stmt) => check_insert(stmt, manager),
        Statement::Delete(ref stmt) => check_delete(stmt, manager),
        Statement::Create(ref stmt) => check_create(stmt, manager),
        Statement::Drop(ref stmt) => check_drop(stmt, manager),
    }
}

pub fn check_select(stmt : &SelectStatement, manager : &TableManager) -> SemResult {
    unimplemented!()
}
pub fn check_update(stmt : &UpdateStatement, manager : &TableManager) -> SemResult {
    unimplemented!()
}
pub fn check_insert(stmt : &InsertStatement, manager : &TableManager) -> SemResult {
    unimplemented!()
}
pub fn check_delete(stmt : &DeleteStatement, manager : &TableManager) -> SemResult {
    unimplemented!()
}

pub fn check_create(stmt : &CreateStatement, manager : &TableManager) -> SemResult {
    try!(check_create_table_exit(stmt, manager));
    try!(check_unique_primary(stmt));
    try!(check_primary_not_null(stmt));
    try!(check_attr_unique(stmt));
    Ok(())
}

pub fn check_create_table_exit(stmt : &CreateStatement, manager : &TableManager) -> SemResult {
    if manager.exist(&stmt.table) {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemTableExist,
            token : dummy_token(),
            error_msg : format!("table {} already exist", &stmt.table),
        })])
    } else {
        Ok(())
    }
}

pub fn check_unique_primary(stmt : &CreateStatement) -> SemResult {
    let primary_attr_list : Vec<String> =
        stmt.decl_list.iter().filter(|d| d.primary).map(|d| d.name.clone()).collect();
    if primary_attr_list.len() == 1 {
        Ok(())
    } else if primary_attr_list.is_empty() {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemNoPrimary,
            token : dummy_token(),
            error_msg : "no primary attribute found".to_string(),
        })])
    } else {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemMultiplePrimary,
            token : dummy_token(),
            error_msg : format!("multiple primary not support: {:?}", primary_attr_list),
        })])
    }
}

pub fn check_primary_not_null(stmt : &CreateStatement) -> SemResult {
    for decl in stmt.decl_list.iter().filter(|d| d.primary && d.nullable) {
        return Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemNullablePrimary,
            token : dummy_token(),
            error_msg : format!("primary attribute can't be null: {}", decl.name),
        })]);
    }
    Ok(())
}

pub fn check_attr_unique(stmt : &CreateStatement) -> SemResult {
    let mut table_set = HashSet::new();
    for name in stmt.decl_list.iter().map(|d| &d.name) {
        if table_set.contains(name) {
            return Err(vec![ErrorRef::new(CompileError{
                error_type : CompileErrorType::SemDuplicateAttr,
                token : dummy_token(),
                error_msg : format!("duplicate attribute name :{}", name),
            })])
        } else {
            table_set.insert(name);
        }
    }
    Ok(())
}

pub fn check_drop(stmt : &DropStatement, manager : &TableManager) -> SemResult {
    if manager.exist(&stmt.table) {
        Ok(())
    } else {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemTableNotExist,
            token : dummy_token(),
            error_msg : table_not_exist(&stmt.table),
        })])
    }
}

pub fn table_not_exist(table : &str) -> String {
    format!("table `{}` not exist", table)
}

pub fn dummy_token() -> TokenRef {
    TokenRef::new(Token{
        column : 0,
        value : "".to_string(),
        token_type : TokenType::UnKnown
    })
}
