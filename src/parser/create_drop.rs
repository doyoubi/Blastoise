use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::compile_error::ErrorList;
use super::common::{
    ValueExpr,
    ValueList,
    consume_next_token_with_type,
    check_parse_to_end,
    exp_list_to_string,
    parse_list_helper,
};


#[derive(Debug)]
pub struct DropStatement {
    pub table : String,
}

impl Display for DropStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "drop table {}", self.table)
    }
}

impl DropStatement {
    pub fn parse(it : &mut TokenIter) -> Result<DropStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Drop));
        try!(consume_next_token_with_type(it, TokenType::Table));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        match check_parse_to_end(it) {
            Some(err) => Err(vec![err]),
            None => Ok(DropStatement{
                table : table_token.value.clone(),
            })
        }
    }
}
