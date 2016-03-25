use std::result::Result;
use ::parser::common::Statement;
use ::parser::compile_error::ErrorList;
use ::parser::lexer::{TokenLine, TokenType};
use ::parser::sem_check::check_sem;
use ::parser::unimpl::check_stmt_unimpl;
use ::store::tuple::TupleData;
use ::store::table::{TableManagerRef, Table, TableSet, AttrType};
use ::exec::gen_plan::{gen_table_set, gen_plan};
use ::exec::gen_plan::gen_proj_info;
use ::exec::error::ExecError;
use ::utils::array::projection;


pub type ResultHandlerRef = Box<ResultHandler>;

pub trait ResultHandler {
    fn handle_error(&mut self, err_msg : String);
    fn handle_tuple_data(&mut self, tuple_data : Option<TupleData>);
    fn handle_non_query_finished(&mut self);
    fn set_tuple_info(&mut self, attr_desc : Vec<AttrType>, attr_index : Vec<usize>);
}


fn gen_parse_result(input : &String) -> Result<Statement, ErrorList> {
    let line = TokenLine::parse(input);
    if line.errors.len() > 0 {
        return Err(line.errors);
    }
    Statement::parse(&mut line.tokens.iter())
}


pub fn process_table_command(input : &String, manager : &TableManagerRef) -> Result<String, ()> {
    match input.as_ref() {
        "show tables" => Ok(show_tables(manager)),
        _ => Err(()),
    }
}

fn show_tables(manager : &TableManagerRef) -> String {
    manager.borrow().show_tables()
}


pub fn sql_handler(input : &String, result_handler : &mut ResultHandler, manager : &TableManagerRef) {
    let parse_result = gen_parse_result(input);
    let mut stmt = match parse_result {
        Ok(stmt) => stmt,
        Err(ref err_list) => return result_handler.handle_error(handle_sql_err(err_list)),
    };
    if let Err(ref err_list) = check_stmt_unimpl(&stmt) {
        return result_handler.handle_error(handle_sql_err(err_list));
    }
    let table_set = gen_table_set(&stmt, manager);
    if let Err(ref err_list) = check_sem(&mut stmt, &table_set) {
        return result_handler.handle_error(handle_sql_err(err_list));
    }

    match &stmt {
        &Statement::Select(..) => {
            let table = get_table(&table_set);
            let mut attr_desc = table.gen_tuple_desc().attr_desc;
            let (attr_index, _) = gen_proj_info(&stmt, &manager);
            attr_desc = projection(&attr_index, attr_desc);
            result_handler.set_tuple_info(attr_desc, attr_index);

            let mut plan = gen_plan(stmt, manager);
            plan.open();
            loop {
                match plan.get_next() {
                    Some(tuple_data) => {
                        result_handler.handle_tuple_data(Some(tuple_data));
                    }
                    None => {
                        if let Some(ref err) = plan.get_error() {
                            result_handler.handle_error(handle_exec_err(err));
                        } else {
                            result_handler.handle_tuple_data(None);
                        }
                        break;
                    }
                }
            }
        }
        _ => {
            let mut plan = gen_plan(stmt, manager);
            plan.open();
            loop {
                match plan.get_next() {
                    Some(..) => continue,
                    None => break,
                }
            }
            if let Some(ref err) = plan.get_error() {
                result_handler.handle_error(handle_exec_err(err));
            } else {
                result_handler.handle_non_query_finished();
                manager.borrow_mut().save_to_file();
            }
        }
    }
}

fn get_table(table_set : &TableSet) -> Table {
    // only suport one table now
    assert_eq!(table_set.tables.len(), 1);
    for (_, t) in table_set.tables.iter() {
        return t.clone();
    }
    panic!("empty table set");
}

fn handle_sql_err(err_list : &ErrorList) -> String {
    let mut err_msg = String::new();
    for err in err_list.iter() {
        let msg = match err.token.token_type {
            TokenType::UnKnown =>
                format!("{:?}: {}", err.error_type, err.error_msg),
            _ => format!("{:?} column {} `{}`: {}",
                err.error_type, err.token.column, err.token.value, err.error_msg),
        };
        err_msg.push_str(&msg);
        err_msg.push('\n');
    }
    err_msg.pop();  // pop the last \n
    err_msg
}

fn handle_exec_err(err : &ExecError) -> String {
    format!("{:?}: {}", err.error_type, err.error_msg)
}
