use std::result::Result;
use ::parser::common::Statement;
use ::parser::compile_error::ErrorList;
use ::parser::lexer::{TokenLine, TokenType};
use ::parser::sem_check::check_sem;
use ::store::tuple::{TupleValue, gen_tuple_value};
use ::store::table::{TableManagerRef, Table, TableSet};
use ::exec::gen_plan::{gen_table_set, gen_plan};
use ::exec::error::ExecError;


pub type ResultHandlerRef = Box<ResultHandler>;

pub trait ResultHandler {
    fn handle_error(&mut self, err_msg : String);
    fn handle_tuple_data(&mut self, tuple_data : Option<Vec<TupleValue>>);
}


fn gen_parse_result(input : &String) -> Result<Statement, ErrorList> {
    let line = TokenLine::parse(input);
    if line.errors.len() > 0 {
        return Err(line.errors);
    }
    Statement::parse(&mut line.tokens.iter())
}


pub fn sql_handler(input : &String, result_handler : &mut ResultHandler, manager : &TableManagerRef) {
    let parse_result = gen_parse_result(input);
    let mut stmt = match parse_result {
        Ok(stmt) => stmt,
        Err(ref err_list) => return result_handler.handle_error(handle_sql_err(err_list)),
    };
    let table_set = gen_table_set(&stmt, manager);
    if let Err(ref err_list) = check_sem(&mut stmt, &table_set) {
        return result_handler.handle_error(handle_sql_err(err_list));
    }

    match &stmt {
        &Statement::Create(..) | &Statement::Drop(..) => {
            let mut plan = gen_plan(stmt, manager);
            plan.open();
            is_match!(plan.get_next(), None);
            if let Some(ref err) = plan.get_error() {
                result_handler.handle_error(handle_exec_err(err));
            } else {
                result_handler.handle_tuple_data(None);
            }
        }
        _ => {
            let mut plan = gen_plan(stmt, manager);
            plan.open();
            let table = get_table(&table_set);
            let tuple_desc = table.gen_tuple_desc();
            loop {
                match plan.get_next() {
                    Some(tuple_data) => {
                        let tuple_value = gen_tuple_value(&tuple_desc, tuple_data);
                        result_handler.handle_tuple_data(Some(tuple_value));
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
            _ => format!("{:?} column{} `{}`: {}",
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
