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
pub struct InsertStatement {
    pub table : String,
    pub value_list : ValueList,
}

impl Display for InsertStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "insert {} values({})", self.table, exp_list_to_string(&self.value_list))
    }
}

impl InsertStatement {
    pub fn parse(it : &mut TokenIter) -> Result<InsertStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Insert));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        try!(consume_next_token_with_type(it, TokenType::Values));
        try!(consume_next_token_with_type(it, TokenType::OpenBracket));
        let value_list = try!(InsertStatement::parse_value_list(it));
        try!(consume_next_token_with_type(it, TokenType::CloseBracket));
        match check_parse_to_end(it) {
            Some(err) => Err(vec![err]),
            None => Ok(InsertStatement{
                table : table_token.value.clone(),
                value_list : value_list,
            })
        }
    }
    pub fn parse_value_list(it : &mut TokenIter) -> Result<ValueList, ErrorList> {
        parse_list_helper(ValueExpr::parse, it)
    }
}
