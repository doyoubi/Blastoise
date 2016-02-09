use std::fmt;
use std::fmt::{Formatter, Display};
use std::vec::Vec;
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::condition::ConditionExpr;
use super::compile_error::ErrorList;
use super::common::{
    ValueExpr,
    consume_next_token_with_type,
    check_parse_to_end,
    seq_parse_helper,
    exp_list_to_string,
    concat_format,
    concat_error_list,
    parse_list_helper,
};


#[derive(Debug)]
pub struct UpdateStatement {
    pub table : String,
    pub set_list : AssignList,
    pub where_condition : Option<ConditionExpr>,
}

impl Display for UpdateStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let mut s = format!("update {} set", self.table);
        s = format!("{} {}", s, exp_list_to_string(&self.set_list));
        s = concat_format(s, "where ", &self.where_condition);
        write!(f, "{}", s)
    }
}

impl UpdateStatement {
    pub fn parse(it : &mut TokenIter) -> Result<UpdateStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Update));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        try!(consume_next_token_with_type(it, TokenType::Set));
        let assign_list = try!(AssignExpr::parse(it));
        let (where_condition, errs) = seq_parse_helper(UpdateStatement::parse_where, it);
        match check_parse_to_end(it) {
            Some(err) => Err(concat_error_list(vec![vec![err], errs])),
            None => Ok(UpdateStatement{
                table : table_token.value.clone(),
                set_list : assign_list,
                where_condition : where_condition,
            })
        }
    }

    pub fn parse_where(it : &mut TokenIter) -> Result<ConditionExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Where));
        ConditionExpr::parse(it)
    }
}

pub type AssignList = Vec<AssignExpr>;

#[derive(Debug)]
pub struct AssignExpr {
    pub attr : String,
    pub value : ValueExpr,
}

impl Display for AssignExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "({} = {})", self.attr, self.value)
    }
}

impl AssignExpr {
    pub fn parse(it : &mut TokenIter) -> Result<AssignList, ErrorList> {
        parse_list_helper(AssignExpr::parse_assign, it)
    }
    pub fn parse_assign(it : &mut TokenIter) -> Result<AssignExpr, ErrorList> {
        let attr_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        try!(consume_next_token_with_type(it, TokenType::EQ));
        let value = try!(ValueExpr::parse(it));
        Ok(AssignExpr{
            attr : attr_token.value.clone(),
            value : value,
        })
    }
}
