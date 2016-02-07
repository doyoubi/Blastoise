use std::vec::Vec;
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::attribute::{AttributeExpr, AttributeList};
use super::condition::ConditionExpr;
use super::compile_error::ErrorList;
use super::common::{
    align_iter,
    get_next_token,
    consume_next_token,
    consume_next_token_with_type,
    consume_next_token_with_type_list,
};

#[derive(Debug)]
pub struct SelectStatement {
    pub select_expr : SelectExpr,
    pub relation_list : Vec<Relation>,
    pub where_condition : Option<ConditionExpr>,
    pub groupby_having : Option<GroupbyHaving>,
    pub order_by_attr : Option<AttributeExpr>,
}

impl SelectStatement {
    pub fn parse(_it : &mut TokenIter) -> Result<SelectStatement, ErrorList> {
        Err(vec![])
    }
}

#[derive(Debug)]
pub enum SelectExpr {
    AllAttribute,
    AttrList(AttributeList),
}

impl SelectExpr {
    pub fn parse(it : &mut TokenIter) -> Result<SelectExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Select));
        let token = try!(get_next_token(it));
        match token.token_type {
            TokenType::Star => Ok(SelectExpr::AllAttribute),
            _ => Ok(SelectExpr::AttrList(try!(AttributeExpr::parse_list(it)))),
        }
    }
}

#[derive(Debug)]
pub enum Relation {
    TableName(String),
    Select(SelectStatement),
}

impl Relation {
    pub fn parse(it : &mut TokenIter) -> Result<Relation, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::From));
        let token = try!(get_next_token(it));
        match token.token_type {
            TokenType::Select => Ok(Relation::Select(try!(SelectStatement::parse(it)))),
            _ => {
                let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
                Ok(Relation::TableName(token.value.clone()))
            }
        }
    }
}

#[derive(Debug)]
pub struct GroupbyHaving {
    pub attr : String,
    pub having_condition : Option<ConditionExpr>,
}

impl GroupbyHaving {
    pub fn parse(it : &mut TokenIter) -> Result<GroupbyHaving, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Having));
        let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        let attr = token.value.clone();
        match GroupbyHaving::parse_having(it) {
            Ok(cond) => Ok(GroupbyHaving{ attr : attr, having_condition : Some(cond) }),
            Err(..) => Ok(GroupbyHaving{ attr : attr, having_condition : None }),
        }
    }

    pub fn parse_having(it : &mut TokenIter) -> Result<ConditionExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Having));
        ConditionExpr::parse(it)
    }
}
