use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::condition::ConditionExpr;
use super::compile_error::ErrorList;
use super::common::{
    consume_next_token_with_type,
    check_parse_to_end,
    seq_parse_helper,
    concat_format,
    concat_error_list,
};


#[derive(Debug)]
pub struct DeleteStatement {
    pub table : String,
    pub where_condition : Option<ConditionExpr>,
}

impl Display for DeleteStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let mut s = format!("delete from {}", self.table);
        s = concat_format(s, "where ", &self.where_condition);
        write!(f, "{}", s)
    }
}

impl DeleteStatement {
    pub fn parse(it : &mut TokenIter) -> Result<DeleteStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Delete));
        try!(consume_next_token_with_type(it, TokenType::From));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        let (where_condition, errs) = seq_parse_helper(DeleteStatement::parse_where, it);
        match check_parse_to_end(it) {
            Some(err) => Err(concat_error_list(vec![vec![err], errs])),
            None => Ok(DeleteStatement{
                table : table_token.value.clone(),
                where_condition : where_condition,
            })
        }
    }
    pub fn parse_where(it : &mut TokenIter) -> Result<ConditionExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Where));
        ConditionExpr::parse(it)
    }
}
